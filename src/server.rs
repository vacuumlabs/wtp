use futures::prelude::*;
use headers::HeaderMapExt;
use hyper::{
    header::{self, AsHeaderName},
    http::HeaderValue,
    Body, HeaderMap, Method, Request, Response, StatusCode,
};
use sea_orm::Database;
use std::sync::RwLock;
use tokio::sync::broadcast;
use tokio_tungstenite::{tungstenite::protocol, WebSocketStream};

pub static WS_BROADCAST_CHANNEL: RwLock<Option<broadcast::Sender<String>>> = RwLock::new(None);

pub fn ws_broadcast(msg: String) {
    WS_BROADCAST_CHANNEL
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .send(msg)
        .ok();
}

use crate::queries;

async fn get_exchange_rates(db_path: String) -> anyhow::Result<String> {
    let db = Database::connect(db_path).await?;
    let data = queries::get_latest_prices(&db).await?;
    Ok(serde_json::to_string(&data)?)
}

fn header_matches<S: AsHeaderName>(headers: &HeaderMap<HeaderValue>, name: S, value: &str) -> bool {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_lowercase() == value)
        .unwrap_or(false)
}

pub fn upgrade_connection(
    req: Request<Body>,
) -> Result<
    (
        Response<String>,
        impl Future<Output = Result<WebSocketStream<hyper::upgrade::Upgraded>, ()>> + Send,
    ),
    Response<String>,
> {
    let mut res = Response::new(String::new());
    let mut header_error = false;

    if !header_matches(req.headers(), header::UPGRADE, "websocket") {
        header_error = true;
    }

    if !header_matches(req.headers(), header::SEC_WEBSOCKET_VERSION, "13") {
        header_error = true;
    }

    if !req
        .headers()
        .typed_get::<headers::Connection>()
        .map(|h| h.contains("Upgrade"))
        .unwrap_or(false)
    {
        header_error = true;
    }

    let key = req.headers().typed_get::<headers::SecWebsocketKey>();

    if key.is_none() {
        header_error = true;
    }

    if header_error {
        *res.status_mut() = StatusCode::BAD_REQUEST;
        return Err(res);
    }

    *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    let h = res.headers_mut();
    h.typed_insert(headers::Upgrade::websocket());
    h.typed_insert(headers::SecWebsocketAccept::from(key.unwrap()));
    h.typed_insert(headers::Connection::upgrade());
    let upgraded = hyper::upgrade::on(req)
        .map_err(|err| tracing::error!("Cannot create websocket: {} ", err))
        .and_then(|upgraded| async {
            let r = WebSocketStream::from_raw_socket(upgraded, protocol::Role::Server, None).await;
            Ok(r)
        });

    Ok((res, upgraded))
}

fn handle_ws_connection(req: Request<Body>) -> hyper::http::Result<Response<String>> {
    let res = match upgrade_connection(req) {
        Err(res) => res,
        Ok((res, ws)) => {
            let run_ws_task = async {
                match ws.await {
                    Ok(mut ws) => {
                        tracing::debug!("Spawning WS");
                        let mut reciever = WS_BROADCAST_CHANNEL
                            .read()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .subscribe();
                        loop {
                            tokio::select! {
                                incoming_message = ws.next() => match incoming_message{
                                    Some(_) => (),
                                    None => break
                                },
                                outgoing_message = reciever.recv() => {ws.send(tokio_tungstenite::tungstenite::Message::Text(outgoing_message.unwrap())).await.ok();}
                            }
                        }
                        tracing::debug!("Closing WS");
                    }
                    Err(_e) => tracing::error!("WS error"),
                }
            };
            tokio::spawn(run_ws_task);
            res
        }
    };
    Ok(res)
}
async fn get_assets(db_path: String) -> anyhow::Result<String> {
    let db = Database::connect(db_path).await?;
    let data = queries::get_assets(&db).await?;
    Ok(serde_json::to_string(&data)?)
}

async fn get_mean_history(
    path: &str,
    query: Option<&str>,
    db_path: String,
) -> anyhow::Result<String> {
    let count = match query {
        Some(query) => {
            let parts: Vec<&str> = query.splitn(2, '=').collect();
            if parts.len() != 2 || parts[0] != "count" {
                return Err(anyhow::anyhow!("Bad query"));
            }
            parts[1].parse::<u64>()?
        }
        None => 10,
    };
    let path: Vec<&str> = path.split('/').collect();
    if path.len() != 4 {
        return Err(anyhow::anyhow!("Bad path"));
    }
    let asset_id1 = path[2].parse::<i64>()?;
    let asset_id2 = path[3].parse::<i64>()?;
    let db = Database::connect(db_path).await?;
    let data = queries::get_token_price_history(asset_id1, asset_id2, count, &db).await?;
    Ok(serde_json::to_string(&data)?)
}

async fn get_swap_history(
    path: &str,
    query: Option<&str>,
    db_path: String,
) -> anyhow::Result<String> {
    let count = match query {
        Some(query) => {
            let parts: Vec<&str> = query.splitn(2, '=').collect();
            if parts.len() != 2 || parts[0] != "count" {
                return Err(anyhow::anyhow!("Bad query"));
            }
            parts[1].parse::<u64>()?
        }
        None => 10,
    };
    let path: Vec<&str> = path.split('/').collect();
    if path.len() != 4 {
        return Err(anyhow::anyhow!("Bad path"));
    }
    let asset_id1 = path[2].parse::<i64>()?;
    let asset_id2 = path[3].parse::<i64>()?;
    let db = Database::connect(db_path).await?;
    let data = queries::get_swap_history(asset_id1, asset_id2, count, &db).await?;
    Ok(serde_json::to_string(&data)?)
}

pub async fn route(req: Request<Body>, db_path: String) -> anyhow::Result<Response<String>> {
    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => Response::builder()
            .header("Content-Type", "application/json")
            .body(String::from("true")),
        (&Method::GET, "/exchange_rates") => Response::builder()
            .header("Content-Type", "application/json")
            .body(get_exchange_rates(db_path).await?),
        (&Method::GET, "/socket") => handle_ws_connection(req),
        (&Method::GET, "/assets") => Response::builder()
            .header("Content-Type", "application/json")
            .body(get_assets(db_path).await?),
        (&Method::GET, path) if path.starts_with("/mean_history/") => Response::builder()
            .header("Content-Type", "application/json")
            .body(get_mean_history(path, req.uri().query(), db_path).await?),
        (&Method::GET, path) if path.starts_with("/asset_swap/") => Response::builder()
            .header("Content-Type", "application/json")
            .body(get_swap_history(path, req.uri().query(), db_path).await?),
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(String::from("404 Not found")),
    };
    Ok(response?)
}
