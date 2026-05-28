use std::error::Error;
use std::sync::{mpsc, Arc, Mutex};
use std::{fs, thread};
use std::time::{Duration, Instant};
use clap::Parser;
use reqwest::blocking::Response;
use pulse::{CheckResult, Config, SiteStatus, Stats};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    urls: String,

    #[arg(short, long, default_value = "5")]
    interval: u64,

    #[arg(short, long, default_value = "3000")]
    timeout: u64,
}
fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
    }
}


fn run() -> Result<(), Box<dyn Error>> {

    let args = Args::parse();
    let config = Config {
        slow_threshold_ms: 500,
        timeout_ms: args.timeout,
    };
    let stat: Arc<Mutex<Stats>> = Arc::new(Mutex::new(Stats::new(0, 0, 0)));
    loop {
        let (tx, rx) = mpsc::channel();
        let urls = get_urls(&args.urls)?;
        for url in urls.clone() {
            let tx = tx.clone();
            let stat_clone = Arc::clone(&stat);
            thread::spawn(move || {
                let site_status= check_url(url.as_str(), config);
                {
                    let mut statistic = stat_clone.lock().unwrap();
                    statistic.total_checks += 1;
                    if let CheckResult::Down(_) = site_status.status {
                        statistic.failures += 1;
                    } else {
                        let successes = statistic.total_checks - statistic.failures;
                        statistic.avg_response =
                            (statistic.avg_response * (successes - 1)
                                + site_status.response_time.unwrap()) / successes;
                    }
                }
                tx.send(site_status).unwrap();
            });
        }
        drop(tx);
        for recv in &rx {
            println!("{}", recv);
        }
        let stat_clone = stat.lock().unwrap();
        println!("{}", stat_clone);
        thread::sleep(Duration::from_secs(args.interval));
    }

    Ok(())
}

fn get_urls(file_path: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let lines = match fs::read_to_string(file_path) {
        Ok(value) => value,
        Err(error) => return Err(error.to_string()),
    };

    for line in lines.lines() {
        result.push(line.to_string());
    }
    Ok(result)
}

fn check_url(url: &str, config: Config) -> SiteStatus {
    let start = Instant::now();

    let response = reqwest::blocking::get(url);
    let response_time = start.elapsed().as_millis();
    site_status_checks(response, response_time as u64, url, config)
}
fn site_status_checks(response: reqwest::Result<Response>, response_time: u64, url: &str, config: Config) -> SiteStatus {
    let status;
    match response {
        Ok(response) => {
            if response.status() != 200 {
                status = CheckResult::Down(format!("{}", response.error_for_status_ref().unwrap_err()));
            } else {
                status = classify(response_time, config);
            }
            SiteStatus::new(response.url().to_string(), Some(response_time), status)
        },
        Err(err) => {
            status = CheckResult::Down(format!("{}", err));
            SiteStatus::new(url.to_string(), None, status)
        }
    }
}


fn classify(response_time: u64, config: Config) -> CheckResult {
    if response_time < config.slow_threshold_ms {
        CheckResult::Ok
    } else if config.slow_threshold_ms < response_time && config.timeout_ms > response_time {
        CheckResult::Slow(response_time)
    } else {
        CheckResult::Down("Timeout".to_string())
    }
}


