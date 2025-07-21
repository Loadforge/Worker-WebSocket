use actix::prelude::*;
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;

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
                        let json = serde_json::to_string_pretty(&config).unwrap_or_else(|_| "Erro ao serializar config".to_string());
                        ctx.text(format!("Recebido:\n{}", json));
                    },
                    Err(_) => {
                        ctx.text("Config inválida");
                    }
                }
            },
            Ok(ws::Message::Close(_)) => {
                ctx.stop();
            },
            _ => {}
        }
    }
}

pub async fn ws_handler(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(WsSession::new(), &req, stream)
}
