use hyper::{Body, Method, Request, Response, StatusCode};
use serde::Serialize;

#[derive(Serialize)]
struct Token {
    name: String,
    policyid: String,
}

#[derive(Serialize)]
struct ExchangeRate {
    first: Token,
    second: Token,
    ratio: f64,
}

fn get_exchange_rates() -> String {
    let mock_data = vec![
        ExchangeRate {
            first: Token {
                name: String::from(""),
                policyid: String::from(""),
            },
            second: Token {
                name: String::from("WingRiders"),
                policyid: String::from("c0ee29a85b13209423b10447d3c2e6a50641a15c57770e27cb9d5073"),
            },
            ratio: 0.344,
        },
        ExchangeRate {
            first: Token {
                name: String::from(""),
                policyid: String::from(""),
            },
            second: Token {
                name: String::from("DANA"),
                policyid: String::from("c88bbd1848db5ea665b1fffbefba86e8dcd723b5085348e8a8d2260f"),
            },
            ratio: 4.32,
        },
    ];
    serde_json::to_string(&mock_data).unwrap()
}

pub async fn handle(req: Request<Body>) -> hyper::http::Result<Response<String>> {
    let builder = Response::builder();

    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => builder
            .header("Content-Type", "text/json")
            .body(String::from("true")),
        (&Method::GET, "/exchange_rates") => builder
            .header("Content-Type", "text/json")
            .body(get_exchange_rates()),
        _ => builder
            .status(StatusCode::NOT_FOUND)
            .body(String::from("404 Not found")),
    };

    response
}
