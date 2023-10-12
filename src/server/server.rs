mod server_config;
use server_config::AppConfig;

use serde_json;

#[macro_use] extern crate rocket;

const SERVER_CONFIG: &str = "resources/server.toml";

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/ledgers")]
async fn list_ledgers() -> String {
    let config = AppConfig::read(SERVER_CONFIG)
        .await
        .expect("failed to read app configuration");

    let ledger_ids: Vec<_> = config.ledgers.keys().collect();
    return serde_json::json!(ledger_ids).to_string();
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