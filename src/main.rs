use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;
use std::{fs, thread};
use std::time::{Instant};
use pulse::{CheckResult, Config, SiteStatus};
const CONFIG: Config = Config {
    slow_threshold_ms: 500,
    timeout_ms: 3000,
};
fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
    }
}


fn run() -> Result<(), Box<dyn Error>> {

    let args: Vec<String> = std::env::args().collect();
    let urls = Vec::from(&args[1..]);

    spawn_checker(urls);

    Ok(())
}


fn spawn_checker(urls: Vec<String>) {
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();
    for url in urls {
        let tx = tx.clone();
        println!("Start thread with: {}", url);
        let handle = thread::spawn(move || {
            let start = Instant::now();
            let mut status;
            let response = reqwest::blocking::get(url.clone());
            let response_time = start.elapsed().as_millis();
            match response {
                Ok(response) => {
                    if response.status() != 200 {
                        status = CheckResult::Down(format!("{}", response.error_for_status_ref().unwrap_err()));
                    }
                    status = classify(response_time as u64);
                    tx.send(SiteStatus::new(response.url().to_string(), Some(response_time as u64), status)).unwrap();
                },
                Err(err) => {
                    status = CheckResult::Down(format!("{}", err));
                    tx.send(SiteStatus::new(url, None, status)).unwrap();
                }
            }

        });

        handles.push(handle);
    }
    drop(tx);
    for result in rx {

        println!("{:}", result_output(&result));
    }

}

fn classify(response_time: u64) -> CheckResult {
    if response_time < CONFIG.slow_threshold_ms {
        CheckResult::Ok
    } else if CONFIG.slow_threshold_ms < response_time && CONFIG.timeout_ms > response_time {
        CheckResult::Slow(response_time)
    } else {
        CheckResult::Down("Timeout".to_string())
    }
}

fn result_output(site_status: &SiteStatus) -> String {
    match site_status.status{
        CheckResult::Ok => format!("✅  {} {}", site_status.url, site_status.response_time.unwrap()),
        CheckResult::Slow(time) => format!("⚠️  {} {} (медленно)", site_status.url, time),
        CheckResult::Down(_) => format!("❌  {} timeout (упал в  {})", site_status.url, site_status.checked_at),
    }
}


