use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    // Set the Unix socket path to connect to the application.
    #[arg(short, long)]
    socket_path: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Trigger a MeiliSearch indexing task on the server.
    Index,
    /// Resume the indexing
    Resume,
    /// Pause the indexing
    Pause,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let socket_path = cli
        .socket_path
        .as_deref()
        .unwrap_or("/tmp/recipe_unix_socket");

    let mut stream = UnixStream::connect(socket_path).await?;

    // TODO: Tracing instead of println.
    println!("Connected to {:?}", socket_path);
    let msg = match &cli.command {
        Some(Commands::Index) => "index",
        Some(Commands::Resume) => "resume",
        Some(Commands::Pause) => "pause",
        _ => "todo",
    };

    stream.write_all(msg.as_bytes()).await?;

    let mut buf = [0; 1024];

    let n = stream.read(&mut buf).await?;

    let response = String::from_utf8_lossy(&buf[..n]);

    println!("Received response: {}", response);

    Ok(())
}
