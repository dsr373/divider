mod server_config;
use server_config::AppConfig;

use rocket::serde::json::Json;

#[macro_use] extern crate rocket;

const SERVER_CONFIG: &str = "resources/server.toml";

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/ledgers")]
async fn list_ledgers() -> Json<Vec<String>> {
    let config = AppConfig::read(SERVER_CONFIG)
        .await
        .expect("failed to read app configuration");

    let ledger_ids: Vec<_> = config.ledgers.keys().map(|k| k.to_owned()).collect();
    return Json(ledger_ids);
}

// #[get("/ledger/<name>")]
// fn list_one_ledger(name: &str) -> String {

// }

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![index, list_ledgers])
        .launch().await?;

    Ok(())
}