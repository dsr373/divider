mod server_config;
use anyhow::Error;
use serde::Deserialize;
use server_config::AppConfig;

mod error;
use error::ServerError;

use divider::{Ledger, backend::{JsonStore, LedgerStore}};

use axum::{
    extract::Path, http::StatusCode, response::{IntoResponse, Json, Response}, routing::{get, post}, Router
};
use tower_http::services::ServeFile;

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
        .ok_or_else(|| ServerError::NotFound(format!("ledger `{}`", name)))?;
    let ledger = JsonStore::new(ledger_path).read()?;
    return Ok(Json(ledger));
}

#[derive(Deserialize)]
struct AddUser{
    name: String
}

async fn add_user_to_ledger(Path(name): Path<String>, Json(add_user): Json<AddUser>) -> Result<Json<Ledger>, ServerError> {
    let config = AppConfig::read(SERVER_CONFIG).await?;

    let ledger_path = config.ledgers.get(&name)
        .ok_or_else(|| ServerError::NotFound(format!("ledger `{}`", name)))?;
    let ledger_store = JsonStore::new(ledger_path);
    let mut ledger = ledger_store.read()?;
    ledger.add_user(&add_user.name);
    ledger_store.save(&ledger)?;

    return Ok(Json(ledger));
}

async fn handle_404() -> Response {
    let body = "Requested resource not found";
    return (StatusCode::NOT_FOUND, body).into_response();
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = Router::new()
        .route("/", get(index))
        .route("/ledgers", get(list_ledgers))
        .route("/ledgers/:name", get(list_one_ledger))
        .route("/ledgers/:name/add-user", post(add_user_to_ledger))
        .route_service("/favicon.ico", ServeFile::new("static/favicon.png"))
        .fallback(handle_404);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}