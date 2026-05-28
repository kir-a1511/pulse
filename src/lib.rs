use chrono::Local;

#[derive(Debug)]
pub struct SiteStatus {
    pub url: String,
    pub response_time: Option<u64>,
    pub status: CheckResult,
    pub checked_at: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CheckResult {
    Ok,
    Slow(u64), // Time in  ms
    Down(String) // Reason
}

#[derive(Clone, Copy)]
pub struct Config {
    pub slow_threshold_ms: u64,
    pub timeout_ms: u64,
}


#[derive(Debug)]
pub struct Stats {
    pub total_checks: u64,
    pub failures: u64,
    pub avg_response: u64,
}

impl Stats {
    pub fn new(total_checks: u64, failures: u64, avg_response: u64) -> Stats {
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
            checked_at: Local::now().time().format("%H:%M:%S").to_string(),
        }
    }
}

impl std::fmt::Display for SiteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let result = match self.status{
            CheckResult::Ok => format!("✅  {} {}", self.url, self.response_time.unwrap()),
            CheckResult::Slow(time) => format!("⚠️  {} {} (slow)", self.url, time),
            CheckResult::Down(_) => format!("❌  {} timeout (drop at {})", self.url, self.checked_at),
        };

        write!(f, "{}", result)
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Total checks: {}\nFailures: {}\nAvg response time: {}", self.total_checks, self.failures, self.avg_response)
    }
}