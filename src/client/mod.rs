use hyper::{Client, Request, Body as HyperBody, Method, Uri, StatusCode};
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use hyper_tls::HttpsConnector;
use crate::models::dsl_model::{DslConfig, Body, Auth, HttpMethod};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use url::Url;
use std::time::Instant;

pub type HttpsClient = Client<HttpsConnector<hyper::client::HttpConnector>>;

pub async fn send_request(
    client: &HttpsClient,
    config: &DslConfig,
) -> Result<(StatusCode, u128), (String, u128)> {
    let mut url = Url::parse(&config.target).map_err(|e| (e.to_string(), 0))?;

    if let Some(params) = &config.query_params {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in params.iter() {
            pairs.append_pair(key, value);
        }
    }

    let uri: Uri = url.as_str()
        .parse::<Uri>()
        .map_err(|e| (e.to_string(), 0))?;

    let method = match config.method {
        HttpMethod::GET     => Method::GET,
        HttpMethod::POST    => Method::POST,
        HttpMethod::PUT     => Method::PUT,
        HttpMethod::DELETE  => Method::DELETE,
        HttpMethod::PATCH   => Method::PATCH,
        HttpMethod::HEAD    => Method::HEAD,
        HttpMethod::OPTIONS => Method::OPTIONS,
    };

    let body = match &config.body {
        Some(Body::Json(json)) => {
            let json_string = serde_json::to_string(json).map_err(|e| (e.to_string(), 0))?;
            HyperBody::from(json_string)
        }
        Some(Body::Xml(xml)) => HyperBody::from(xml.clone()),
        None => HyperBody::empty(),
    };

    let mut req_builder = Request::builder()
        .method(method)
        .uri(uri.clone());

    match &config.body {
        Some(Body::Json(_)) => {
            req_builder = req_builder.header(CONTENT_TYPE, "application/json");
        }
        Some(Body::Xml(_)) => {
            req_builder = req_builder.header(CONTENT_TYPE, "application/xml");
        }
        None => {}
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
            Auth::ApiKey { key_name, key_value, in_header: true } => {
                req_builder = req_builder.header(key_name, key_value);
            }
            Auth::ApiKey { in_header: false, .. } => {
            }
            Auth::None => {}
        }
    }

    let request = req_builder.body(body).map_err(|e| (e.to_string(), 0))?;

    let start = Instant::now();
    let response = client.request(request).await;
    let duration = start.elapsed().as_millis();

    match response {
        Ok(resp) => Ok((resp.status(), duration)),
        Err(e) => {
            let msg = if e.is_connect() {
                "Connection refused or host unreachable"
            } else if e.is_timeout() {
                "Timeout"
            } else if e.is_closed() {
                "Connection closed unexpectedly"
            } else {
                "Unknown network error"
            };
            Err((msg.to_string(), duration))
        }
    }
}
