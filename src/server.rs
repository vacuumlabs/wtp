use hyper::{Body, Method, Request, Response, StatusCode};
use sea_orm::Database;

use crate::queries;

async fn get_exchange_rates(db_path: String) -> anyhow::Result<String> {
    let db = Database::connect(db_path).await?;
    let data = queries::get_latest_prices(&db).await?;
    Ok(serde_json::to_string(&data)?)
}

pub async fn handle(req: Request<Body>, db_path: String) -> anyhow::Result<Response<String>> {
    let builder = Response::builder();

    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => builder
            .header("Content-Type", "application/json")
            .body(String::from("true")),
        (&Method::GET, "/exchange_rates") => builder
            .header("Content-Type", "application/json")
            .body(get_exchange_rates(db_path).await?),
        _ => builder
            .status(StatusCode::NOT_FOUND)
            .body(String::from("404 Not found")),
    };

    Ok(response?)
}
