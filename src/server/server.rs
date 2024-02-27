mod server_config;
use server_config::AppConfig;

use divider::{Ledger, backend::{JsonStore, LedgerStore}};

use log::error;
use rocket::serde::json::Json;
use rocket::fs::NamedFile;

#[macro_use] extern crate rocket;

// TODO: can we have global config read only once? probably not
const SERVER_CONFIG: &str = "resources/server.toml";
async fn read_config() -> Option<AppConfig> {
    AppConfig::read(SERVER_CONFIG).await
        .map_err(|err| error!("{}", err)).ok()
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/ledgers")]
async fn list_ledgers() -> Option<Json<Vec<String>>> {
    let config = read_config().await?;

    let ledger_ids: Vec<_> = config.ledgers.keys().map(|k| k.to_owned()).collect();
    return Some(Json(ledger_ids));
}

// TODO: make the return type Result<Json<Ledger>> instead so we can differentiate 404 from 500
#[get("/ledger/<name>")]
async fn list_one_ledger(name: &str) -> Option<Json<Ledger>> {
    let config = read_config().await?;

    return config.ledgers.get(name)
        .map(|path| JsonStore::new(path))
        .and_then(|store| store.read().map_err(|err| error!("{}", err)).ok())
        .map(|ledger| Json(ledger));
}

#[get("/favicon.ico")]
async fn favicon() -> Option<NamedFile> {
    NamedFile::open("static/favicon.png").await.ok()
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![index, list_ledgers, list_one_ledger, favicon])
        .launch().await?;

    Ok(())
}