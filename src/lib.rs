use std::time::Instant;

#[derive(Debug)]
pub struct SiteStatus {
    pub url: String,
    pub response_time: Option<u64>,
    pub status: CheckResult,
    pub checked_at: String,
}

#[derive(Debug)]
pub enum CheckResult {
    Ok,
    Slow(u64), // Time in  ms
    Down(String) // Reason
}

pub struct Config {
    pub slow_threshold_ms: u64,
    pub timeout_ms: u64,
}
struct Stats {
    total_checks: u64,
    failures: u64,
    avg_response: f64,
}

impl Stats {
    fn new(total_checks: u64, failures: u64, avg_response: f64) -> Stats {
        Stats {
            total_checks,
            failures,
            avg_response,
        }
    }
}

impl SiteStatus {
    pub fn new(url: String, response_time: Option<u64>, status: CheckResult) -> SiteStatus {
        SiteStatus {
            url,
            response_time,
            status,
            checked_at: String::from(format!("{:?}", Instant::now())),
        }
    }
}