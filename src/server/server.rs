mod server_config;
use anyhow::Error;
use server_config::AppConfig;

mod error;
use error::ServerError;

use divider::{Ledger, backend::{JsonStore, LedgerStore}};

use axum::{
    response::Json,
    routing::get,
    extract::Path,
    Router
};

// TODO: can we have global config read only once? probably not
const SERVER_CONFIG: &str = "resources/server.toml";

async fn index() -> &'static str {
    "Hello, world!"
}

async fn list_ledgers() -> Result<Json<Vec<String>>, ServerError> {
    let config = AppConfig::read(SERVER_CONFIG).await?;

    let ledger_ids: Vec<_> = config.ledgers.keys().map(|k| k.to_owned()).collect();
    return Ok(Json(ledger_ids));
}

async fn list_one_ledger(Path(name): Path<String>) -> Result<Json<Ledger>, ServerError> {
    let config = AppConfig::read(SERVER_CONFIG).await?;

    let ledger_path = config.ledgers.get(&name)
        .ok_or_else(|| ServerError::NotFound(format!("{}", name)))?;
    let ledger = JsonStore::new(ledger_path).read()?;
    return Ok(Json(ledger));
}

// #[get("/favicon.ico")]
// async fn favicon() -> Option<NamedFile> {
//     NamedFile::open("static/favicon.png").await.ok()
// }

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = Router::new()
        .route("/", get(index))
        .route("/ledgers", get(list_ledgers))
        .route("/ledgers/:name", get(list_one_ledger));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}