use actix::prelude::*;
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use std::collections::HashMap;
use std::env;
use crate::utils::hardware::get_hardware_info;
use crate::models::dsl_model::DslConfig;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tokio::task;
use tokio::time::sleep;
use tokio_stream::wrappers::UnboundedReceiverStream;
use chrono::Local;
use hyper_tls::HttpsConnector;
use hyper::Client;

use crate::client::{send_request, HttpsClient};
use crate::models::metrics::Metrics;

use std::sync::atomic::{AtomicUsize};

pub struct WsSession {
    tx: Option<mpsc::UnboundedSender<String>>,
}

impl WsSession {
    pub fn new() -> Self {
        Self { tx: None }
    }
}
static ACTIVE_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);
const MAX_CONNECTIONS: usize = 1;

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
    let current = ACTIVE_CONNECTIONS.load(Ordering::SeqCst);

    if current >= MAX_CONNECTIONS {
        ctx.close(Some(ws::CloseReason {
            code: ws::CloseCode::Policy,
            description: Some("Maximum number of simultaneous connections reached".to_string()),
        }));
        ctx.stop();
    } else {
        let new_total = ACTIVE_CONNECTIONS.fetch_add(1, Ordering::SeqCst) + 1;
        println!("WebSocket connection started, active connections: {}", new_total);

        let (tx, rx) = mpsc::unbounded_channel::<String>();
        self.tx = Some(tx.clone());

        let stream = UnboundedReceiverStream::new(rx);
        ctx.add_stream(stream);
    }
}

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let new_total = ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::SeqCst).saturating_sub(1);
        println!("WebSocket connection closed, active connections: {}", new_total);
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

impl StreamHandler<String> for WsSession {
    fn handle(&mut self, msg: String, ctx: &mut Self::Context) {
        ctx.text(msg);
    }
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
                                let tx = match &self.tx {
                                    Some(sender) => sender.clone(),
                                    None => {
                                        ctx.text(serde_json::json!({
                                            "status": "error",
                                            "message": "Canal interno não inicializado"
                                        }).to_string());
                                        return;
                                    }
                                };
                               
                                ctx.text(serde_json::json!({
                                    "status": "start-config",
                                    "config": {
                                        "name": config.name,
                                        "target": config.target,
                                        "method": format!("{:?}", config.method),
                                        "concurrency": config.concurrency,
                                        "duration": config.duration,
                                        "auth": config.auth.as_ref().map(|a| format!("{:?}", a)).unwrap_or_else(|| "None".to_string()),
                                        "body": config.body.as_ref().map(|b| format!("{:?}", b)),
                                        "query_params": config.query_params,
                                        "headers": config.headers,
                                        "hardware_info": {
                                            "cpu_cores": cpu_cores,
                                            "total_ram_mb": total_mem_kb as f64 / 1024.0,
                                            "free_ram_mb": free_mem_kb as f64 / 1024.0,
                                        }
                                    }
                                }).to_string());

                                let https = HttpsConnector::new();
                                let client: HttpsClient = Client::builder().build::<_, hyper::Body>(https);
                                let client = Arc::new(client);
                                let config = Arc::new(config);
                                

                                let metrics = Arc::new(Mutex::new(Metrics {
                                    fastest_response: f64::MAX,
                                    slowest_response: f64::MIN,
                                    status_counts: HashMap::new(),
                                    ..Default::default()
                                }));


                                let response_times = Arc::new(Mutex::new(Vec::new()));
                                let running = Arc::new(AtomicBool::new(true));
                                let mut handles = Vec::new();


                                let duration_secs = config.duration;
                                let end_time = Instant::now() + Duration::from_secs(duration_secs);


                                for _ in 0..config.concurrency {
                                    let client = Arc::clone(&client);
                                    let config = Arc::clone(&config);
                                    let metrics = Arc::clone(&metrics);
                                    let response_times = Arc::clone(&response_times);
                                    let running = Arc::clone(&running);
                                    let tx = tx.clone();

                                    let handle = task::spawn(async move {
                                        while running.load(Ordering::Relaxed) && Instant::now() < end_time &&  ACTIVE_CONNECTIONS.load(Ordering::SeqCst) > 0   {
                                            let result = send_request(&client, &config).await;

                                            match result {
                                                Ok((status, duration)) => {
                                                    let elapsed = duration as f64;

                                                    {
                                                        let mut rt = response_times.lock().unwrap();
                                                        rt.push(elapsed);
                                                    }

                                                    let mut m = metrics.lock().unwrap();
                                                    m.total_requests += 1;
                                                    m.successful_requests += 1;
                                                    m.total_duration += elapsed;

                                                    let status_code = status.as_u16();
                                                    let message = serde_json::json!({
                                                        "status": "process",
                                                        "http_status": status_code,
                                                        "duration_ms": elapsed,
                                                    });
                                                    let _ = tx.send(message.to_string());

                                                    let key = status_code.to_string();
                                                    *m.status_counts.entry(key).or_insert(0) += 1;

                                                    if elapsed < m.fastest_response {
                                                        m.fastest_response = elapsed;
                                                    }
                                                    if elapsed > m.slowest_response {
                                                        m.slowest_response = elapsed;
                                                    }
                                                }
                                                Err((err_msg, duration)) => {
                                                    let elapsed = duration as f64;

                                                    {
                                                        let mut rt = response_times.lock().unwrap();
                                                        rt.push(elapsed);
                                                    }

                                                    let mut m = metrics.lock().unwrap();
                                                    m.total_requests += 1;
                                                    m.failed_requests += 1;
                                                    m.total_duration += elapsed;

                                                    let message = serde_json::json!({
                                                        "status": "process",
                                                        "http_status": "REQUEST_ERROR",
                                                        "error": err_msg,
                                                        "duration_ms": elapsed,
                                                    });
                                                    let _ = tx.send(message.to_string());

                                                    *m.status_counts.entry("REQUEST_ERROR".to_string()).or_insert(0) += 1;

                                                    if elapsed < m.fastest_response {
                                                        m.fastest_response = elapsed;
                                                    }
                                                    if elapsed > m.slowest_response {
                                                        m.slowest_response = elapsed;
                                                    }
                                                }
                                            }
                                        }
                                    });

                                    handles.push(handle);
                                }

                                let metrics = Arc::clone(&metrics);
                                let response_times = Arc::clone(&response_times);
                                let config = Arc::clone(&config);

                                task::spawn(async move {
                                    sleep(Duration::from_secs(duration_secs)).await;
                                    running.store(false, Ordering::Relaxed);

                                    for handle in handles {
                                        let _ = handle.await;
                                    }

                                    let mut final_metrics = metrics.lock().unwrap();
                                    let response_times = response_times.lock().unwrap();

                                    let median = calculate_median(&response_times);
                                    let total_time_secs = duration_secs as f64;
                                    let throughput = final_metrics.total_requests as f64 / total_time_secs;

                                    final_metrics.target_url = config.target.clone();
                                    final_metrics.http_method = format!("{:?}", config.method);
                                    final_metrics.duration_secs = config.duration;
                                    final_metrics.concurrency = config.concurrency;
                                    final_metrics.throughput = throughput;
                                    final_metrics.median_response_time = median;
                                    final_metrics.timestamp = Local::now().format("%Y/%m/%d %H:%M:%S").to_string();

                                    let final_metrics_msg = serde_json::json!({
                                        "status": "final_metrics",
                                        "target_url": final_metrics.target_url,
                                        "http_method": final_metrics.http_method,
                                        "duration_secs": final_metrics.duration_secs,
                                        "concurrency": final_metrics.concurrency,
                                        "timestamp": final_metrics.timestamp,
                                        "total_requests": final_metrics.total_requests,
                                        "successful_requests": final_metrics.successful_requests,
                                        "failed_requests": final_metrics.failed_requests,
                                        "fastest_response_ms": final_metrics.fastest_response,
                                        "slowest_response_ms": final_metrics.slowest_response,
                                        "median_response_ms": final_metrics.median_response_time,
                                        "throughput_rps": final_metrics.throughput,
                                        "status_counts": final_metrics.status_counts,
                                    });

                                    let _ = tx.send(final_metrics_msg.to_string());
                                    
                                });
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

fn calculate_median(data: &[f64]) -> f64 {
    let mut sorted = data.to_owned();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let len = sorted.len();
    if len == 0 {
        return 0.0;
    }
    if len % 2 == 0 {
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
    } else {
        sorted[len / 2]
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
            if ACTIVE_CONNECTIONS.load(Ordering::SeqCst) >= MAX_CONNECTIONS {
                return Ok(HttpResponse::TooManyRequests().body("There is already an active WebSocket connection"));
            }
            let res = ws::start(WsSession::new(), &req, stream);
            res
        }
        _ => Ok(HttpResponse::Unauthorized().finish()),
    }
}