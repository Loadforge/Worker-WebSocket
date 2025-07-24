use actix::prelude::*;
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use std::collections::HashMap;
use std::env;
use crate::utils::hardware::get_hardware_info;
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

fn validate_config(config: &DslConfig) -> Result<(), String> {
    let (cpu_cores, _total_mem_kb, free_mem_kb) = get_hardware_info();

    let min_ram_kb = 500 * 1024;
    let ram_per_thread_kb = 50 * 1024;

    if free_mem_kb < min_ram_kb {
        return Err(format!("Insufficient free RAM: {:.2} MB", free_mem_kb as f64 / 1024.0));
    }

    if config.concurrency > cpu_cores * 3 {
        return Err(format!(
            "Concurrency {} is too high for CPU cores {}",
            config.concurrency, cpu_cores
        ));
    }

    if (config.concurrency as u64) * ram_per_thread_kb > free_mem_kb {
        return Err(format!(
            "Concurrency {} requires more RAM than available. Required: {:.2} MB, Available: {:.2} MB",
            config.concurrency,
            (config.concurrency as u64 * ram_per_thread_kb) as f64 / 1024.0,
            free_mem_kb as f64 / 1024.0
        ));
    }

    Ok(())
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<DslConfig>(&text) {
                    Ok(config) => {
                        let (cpu_cores, total_mem_kb, free_mem_kb) = get_hardware_info();

                        match validate_config(&config) {
                            Ok(()) => {
                                ctx.text(serde_json::json!({
                                    "status": "start-config",
                                    "config": {
                                        "name": config.name,
                                        "target": config.target,
                                        "method": format!("{:?}", config.method),
                                        "concurrency": config.concurrency,
                                        "duration": config.duration,
                                        "timeout_ms": config.timeout.unwrap_or_default(),
                                        "auth": config.auth.as_ref().map(|a| format!("{:?}", a)).unwrap_or_else(|| "None".to_string()),
                                        "body": config.body.as_ref().map(|b| format!("{:?}", b)),
                                        "query_params": config.query_params,
                                        "hardware_info": {
                                            "cpu_cores": cpu_cores,
                                            "total_ram_mb": total_mem_kb as f64 / 1024.0,
                                            "free_ram_mb": free_mem_kb as f64 / 1024.0,
                                        }
                                    }
                                }).to_string());
                            }
                            Err(err_msg) => {
                                ctx.text(serde_json::json!({
                                    "status": "error",
                                    "message": err_msg
                                }).to_string());
                            }
                        }
                    }
                    Err(_) => {
                        ctx.text(serde_json::json!({
                            "status": "error",
                            "message": "Invalid config format"
                        }).to_string());
                    }
                }
            }
            Ok(ws::Message::Close(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
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
        _ => Ok(HttpResponse::Unauthorized().finish()),
    }
}
