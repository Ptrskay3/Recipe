#[macro_use] extern crate rocket;

#[get("/hello")]
async fn hello() -> String {
    "Hello world".into()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello])
}
