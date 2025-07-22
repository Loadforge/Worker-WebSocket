use actix::prelude::*;
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use std::collections::HashMap;
use std::env;

use crate::models::dsl_model::DslConfig;

pub struct WsSession;

impl WsSession {
    pub fn new() -> Self {
        Self
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("WebSocket connection started");
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<DslConfig>(&text) {
                    Ok(config) => {
                        let json = serde_json::to_string_pretty(&config)
                            .unwrap_or_else(|_| "Failed to serialize config".to_string());
                        ctx.text(format!("Received:\n{}", json));
                    }
                    Err(_) => {
                        ctx.text("Invalid config");
                    }
                }
            }
            Ok(ws::Message::Close(_)) => {
                ctx.stop();
            }
            _ => {}
        }
    }
}

pub async fn ws_handler(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let expected_token = env::var("WS_SECRET_TOKEN").unwrap_or_default();
    let query_params: HashMap<_, _> = req
        .query_string()
        .split('&')
        .filter_map(|pair| {
            let mut iter = pair.splitn(2, '=');
            match (iter.next(), iter.next()) {
                (Some(k), Some(v)) => Some((k.to_string(), v.to_string())),
                _ => None,
            }
        })
        .collect();


    match query_params.get("token") {
        Some(token) if token == &expected_token => {
            ws::start(WsSession::new(), &req, stream)
        }
        _ => {
            Ok(HttpResponse::Unauthorized().finish())
        }
    }
}
