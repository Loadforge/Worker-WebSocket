use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Metrics {
    pub target_url: String,
    pub http_method: String,
    pub duration_secs: u64,
    pub concurrency: u64,

    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,

    pub fastest_response: f64, 
    pub slowest_response: f64,
    pub median_response_time: f64,

    pub total_duration: f64, 
    pub throughput: f64,     

    pub timestamp: String,

    pub status_counts: HashMap<String, u64>,
}
