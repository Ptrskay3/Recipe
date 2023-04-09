use anyhow::Context;
use axum::{
    body::Bytes,
    extract::{BodyStream, DefaultBodyLimit, Multipart, Path},
    middleware::from_extractor_with_state,
    routing::post,
    BoxError, Router,
};
use futures::{Stream, TryStreamExt};
use sqlx::{Acquire, PgExecutor};
use std::io::{self, ErrorKind};
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tower_http::limit::RequestBodyLimitLayer;

use crate::{
    error::ApiError,
    extractors::{DatabaseConnection, Uploader},
    routes::admin::AdminUser,
    state::AppState,
};

pub const UPLOADS_DIRECTORY: &str = "uploads";

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/:file_name", post(save_request_body))
        .route_layer(from_extractor_with_state::<AdminUser, _>(state))
        .route("/", post(accept_form))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(25 * 1024 * 1024)) // 25mb
}

pub async fn save_request_body(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(file_name): Path<String>,
    uploader: Uploader,
    body: BodyStream,
) -> Result<(), ApiError> {
    stream_to_file(&file_name, uploader.id, body, &mut conn).await
}

// Handler that accepts a multipart form upload and streams each field to a file.
pub async fn accept_form(
    uploader: Uploader,
    DatabaseConnection(mut conn): DatabaseConnection,
    mut multipart: Multipart,
) -> Result<(), ApiError> {
    let mut tx = conn.begin().await?;
    while let Some(field) = multipart
        .next_field()
        .await
        .context("Failed to get next field")?
    {
        let Some(file_name) = field.file_name().map(ToOwned::to_owned) else { continue };

        stream_to_file(&file_name, uploader.id, field, &mut tx).await?;
    }

    tx.commit().await?;

    Ok(())
}

async fn stream_to_file<'c, T, S, E, P>(
    path: P,
    user_id: uuid::Uuid,
    stream: S,
    tx: T,
) -> Result<(), ApiError>
where
    T: PgExecutor<'c>,
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
    P: AsRef<std::path::Path>,
{
    if !path_is_valid(&path) {
        return Err(ApiError::BadRequest);
    }

    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the necessary directories and the file.
        let user_dir = std::path::Path::new(UPLOADS_DIRECTORY).join(user_id.to_string());
        if let Err(e) = tokio::fs::create_dir_all(user_dir.clone()).await {
            tracing::error!(
                "User directory doesn't exist and couldn't be crated: {:?}",
                e
            );
        }
        let file_path = user_dir.join(path);
        let mut file = BufWriter::new(File::create(file_path.clone()).await?);

        // Copy the body into the file.
        let bytes_copied = tokio::io::copy(&mut body_reader, &mut file).await?;
        tracing::info!("written {bytes_copied} bytes");

        sqlx::query!(
            "INSERT INTO uploads (uploader_id, bytes, file_name) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            user_id,
            bytes_copied as f32,
            file_path
                .as_path()
                .file_name()
                .expect("only valid files can be uploaded")
                .to_str()
                .expect("only valid files can be uploaded")
        )
        .execute(tx)
        .await
        .map_err(|_| io::Error::new(ErrorKind::Interrupted, "failed to insert to uploads table"))?;
        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|_| ApiError::BadRequest)
}

// to prevent directory traversal attacks we ensure the path consists of exactly one normal
// component
fn path_is_valid<P: AsRef<std::path::Path>>(path: P) -> bool {
    let mut components = path.as_ref().components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}
