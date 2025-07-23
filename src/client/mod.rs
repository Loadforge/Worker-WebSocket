use hyper::{Client, Request, Body as HyperBody, Method, Uri, StatusCode};
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use hyper_tls::HttpsConnector;
use crate::models::dsl_model::{DslConfig, Body, Auth, HttpMethod};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use url::Url;
use std::time::Instant;
use colored::*;

pub type HttpsClient = Client<HttpsConnector<hyper::client::HttpConnector>>;

pub async fn send_request(
    client: &HttpsClient,
    config: &DslConfig,
) -> Result<StatusCode, Box<dyn std::error::Error + Send + Sync>> {
    let mut url = Url::parse(&config.target)?;

    if let Some(params) = &config.query_params {
        let mut pairs = url.query_pairs_mut();
        for (k, v) in params {
            pairs.append_pair(k, v);
        }
    }

    let uri: Uri = url.as_str().parse()?;

    let method = match config.method {
        HttpMethod::GET => Method::GET,
        HttpMethod::POST => Method::POST,
        HttpMethod::PUT => Method::PUT,
        HttpMethod::DELETE => Method::DELETE,
        HttpMethod::PATCH => Method::PATCH,
        HttpMethod::HEAD => Method::HEAD,
        HttpMethod::OPTIONS => Method::OPTIONS,
    };

    let body = match &config.body {
        Some(Body::Json(value)) => HyperBody::from(serde_json::to_string(value)?),
        Some(Body::Xml(s)) => HyperBody::from(s.clone()),
        None => HyperBody::empty(),
    };

    let mut req_builder = Request::builder()
        .method(method)
        .uri(uri.clone());

    if let Some(Body::Json(_)) = &config.body {
        req_builder = req_builder.header(CONTENT_TYPE, "application/json");
    } else if let Some(Body::Xml(_)) = &config.body {
        req_builder = req_builder.header(CONTENT_TYPE, "application/xml");
    }

    if let Some(auth) = &config.auth {
        match auth {
            Auth::Basic { username, password } => {
                let encoded = BASE64.encode(format!("{}:{}", username, password));
                req_builder = req_builder.header(AUTHORIZATION, format!("Basic {}", encoded));
            }
            Auth::Bearer { token } => {
                req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", token));
            }
            Auth::ApiKey {
                key_name,
                key_value,
                in_header,
            } => {
                if *in_header {
                    req_builder = req_builder.header(key_name, key_value);
                }
            }
            Auth::None => {}
        }
    }

    let request = req_builder.body(body)?;

    let start = Instant::now();
    let result = client.request(request).await;
    let duration = start.elapsed();

    match result {
        Ok(response) => {
            let status = response.status();
            println!(
                "{} {} {} {}",
                "status :".green().bold(),
                status.as_u16().to_string().bold(),
                "| duration :".blue().bold(),
                format!("{}ms", duration.as_millis()).bold(),
            );
            Ok(status) 
        }
        Err(e) => {
            let error_reason = if e.is_connect() {
                "Network Error (Connection refused or host unreachable)"
            } else if e.is_timeout() {
                "Network Error (Timeout)"
            } else if e.is_closed() {
                "Network Error (Connection closed unexpectedly)"
            } else {
                "Network Error (Unknown)"
            };

            eprintln!(
                "{} {} {} {}",
                "status :".red().bold(),
                error_reason.red().bold(),
                "| duration :".blue().bold(),
                format!("{}ms", duration.as_millis()).bold(),
            );

            Err(Box::new(e)) 
        }
    }
}
