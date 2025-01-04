use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::{
    str::FromStr,
    thread,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use clap::{Parser, ValueEnum};
use reqwest::header::USER_AGENT;
use reqwest::{blocking::Client, Method, StatusCode};

/// Command line arguments parser
#[derive(Parser)]
#[clap(about = "Loadster is a simple load testing tool that allows you to test the performance of your web applications by sending concurrent HTTP requests.")]
struct Cli {
    /// The target URL for the load test
    #[clap(short = 'u', long, help = "The target URL for the load test")]
    url: String,

    /// The HTTP method to use (default: GET). Supported methods: GET, POST, PUT, DELETE, PATCH
    #[clap(short = 'm', long, value_enum, default_value_t = HttpMethod::Get, help = "The HTTP method to use (default: GET). Supported methods: GET, POST, PUT, DELETE, PATCH")]
    method: HttpMethod,

    /// The number of concurrent users (default: 10)
    #[clap(short = 'c', long, default_value = "10", help = "The number of concurrent users (default: 10)")]
    users: i32,

    /// The timeout for each request in seconds (default: 30)
    #[clap(short = 't', long, default_value = "30", help = "The timeout for each request in seconds (default: 30)")]
    timeout: u64,

    /// Additional headers to include in the requests
    #[clap(short = 'H', long, help = "Additional headers to include in the requests")]
    headers: Vec<String>,

    /// The body of the request (for POST, PUT methods)
    #[clap(short = 'b', long, help = "The body of the request (for POST, PUT methods)")]
    body: Option<String>,

    /// Enable verbose output
    #[clap(short = 'v', long, help = "Enable verbose output")]
    verbose: bool,

    /// Save the results to a file
    #[clap(short = 'o', long, help = "Save the results to a file")]
    output: Option<String>,
}

/// Supported HTTP methods
#[derive(Debug, Clone, ValueEnum)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "get" => Ok(HttpMethod::Get),
            "post" => Ok(HttpMethod::Post),
            "put" => Ok(HttpMethod::Put),
            "delete" => Ok(HttpMethod::Delete),
            "patch" => Ok(HttpMethod::Patch),
            _ => Err(format!("'{}' is not a valid HTTP method", s)),
        }
    }
}

impl From<HttpMethod> for Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Patch => Method::PATCH,
        }
    }
}

/// Struct to hold response details
#[derive(Debug, Clone)]
struct ResponseDetails {
    status: StatusCode,
    time: u64,      // Time in milliseconds
    timestamp: u64, // Timestamp in seconds since UNIX_EPOCH
}

fn main() {
    let args = Cli::parse();
    let data = call_api(
        args.url.to_owned(),
        args.method.into(),
        args.users,
        args.timeout,
        args.headers,
        args.body,
        args.verbose,
    )
    .unwrap();
    display_results(&data);
    generate_report(&data, &args.url);

    if let Some(output) = args.output {
        save_results(&data, &output);
    }
}

/// Function to call the API concurrently
fn call_api(
    url: String,
    method: Method,
    concurrency: i32,
    timeout: u64,
    headers: Vec<String>,
    body: Option<String>,
    verbose: bool,
) -> Result<Vec<ResponseDetails>, reqwest::Error> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(timeout))
        .build()?;
    let data = Arc::new(Mutex::new(vec![]));
    let mut handles = vec![];

    for i in 0..concurrency {
        let url = url.clone();
        let data = Arc::clone(&data);
        let client = client.clone();
        let method = method.clone();
        let headers = headers.clone();
        let body = body.clone();
        let handle = thread::spawn(move || {
            let start = Instant::now();
            let mut request = client
                .request(method.clone(), url.as_str())
                .header(USER_AGENT, "loadster 1.0.0");

            for header in headers {
                let parts: Vec<&str> = header.splitn(2, ':').collect();
                if parts.len() == 2 {
                    request = request.header(parts[0], parts[1]);
                }
            }

            if let Some(body) = body {
                request = request.body(body.clone());
            }

            let res = request.send();
            match res {
                Ok(res) => {
                    let duration = start.elapsed();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    if verbose {
                        println!("i: {} ,Status: {}", i, res.status());
                    }
                    let mut data = data.lock().unwrap();
                    let response_details = ResponseDetails {
                        status: res.status(),
                        time: duration.as_millis() as u64,
                        timestamp,
                    };
                    data.push(response_details);
                }
                Err(e) => {
                    eprintln!("Request failed: {}", e);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = {
        let data = data.lock().unwrap();
        data.clone()
    };
    Ok(result)
}

/// Function to display the results of the load test
fn display_results(data: &[ResponseDetails]) {
    let total_requests = data.len();
    let successful_requests = data.iter().filter(|d| d.status.is_success()).count();
    let failed_requests = total_requests - successful_requests;
    let total_time: u64 = data.iter().map(|d| d.time).sum();
    let avg_time = total_time as f64 / total_requests as f64;

    // Calculate additional metrics
    let mut times: Vec<u64> = data.iter().map(|d| d.time).collect();
    times.sort_unstable();
    let median_time = times[times.len() / 2];
    let min_time = times.first().unwrap_or(&0); // Prefix with underscore
    let max_time = times.last().unwrap_or(&0);

    println!("\nLoad Test Results:");
    println!("Total Requests: {}", total_requests);
    println!("Successful Requests: {}", successful_requests);
    println!("Failed Requests: {}", failed_requests);
    println!("Total Time: {} ms", total_time);
    println!("Average Time per Request: {:.2} ms", avg_time);
    println!("Median Time: {} ms", median_time);
    println!("Minimum Time: {} ms", min_time);
    println!("Maximum Time: {} ms", max_time);
}

/// Function to generate a detailed load test report
fn generate_report(data: &[ResponseDetails], url: &str) {
    let total_requests = data.len();
    let successful_requests = data.iter().filter(|d| d.status.is_success()).count();
    let failed_requests = total_requests - successful_requests;
    let total_time: u64 = data.iter().map(|d| d.time).sum();
    let avg_time = total_time as f64 / total_requests as f64;

    // Calculate additional metrics
    let mut times: Vec<u64> = data.iter().map(|d| d.time).collect();
    times.sort_unstable();
    let median_time = times[times.len() / 2];
    let max_time = times.last().unwrap_or(&0);
    let p95_time = times[(times.len() as f64 * 0.95) as usize];
    let p99_time = times[(times.len() as f64 * 0.99) as usize];

    // Calculate response code distribution
    let mut response_codes: HashMap<StatusCode, usize> = HashMap::new();
    for detail in data {
        *response_codes.entry(detail.status).or_insert(0) += 1;
    }

    // Calculate throughput
    let duration_seconds = total_time as f64 / 1000.0;
    let throughput = total_requests as f64 / duration_seconds;

    println!("\nLoad Test Report");
    println!("Summary");
    println!("Metric\tValue");
    println!("Target URL\t{}", url);
    println!("Total Requests\t{}", total_requests);
    println!("Successful Requests\t{}", successful_requests);
    println!("Failed Requests\t{}", failed_requests);
    println!("Duration\t{:.2} seconds", duration_seconds);
    println!("Throughput\t{:.2} req/s", throughput);
    println!("Avg Latency\t{:.2} ms", avg_time);
    println!("P95 Latency\t{} ms", p95_time);
    println!("P99 Latency\t{} ms", p99_time);

    println!("\nResponse Codes");
    println!("Code\tCount\tPercentage");
    for (code, count) in &response_codes {
        println!(
            "{}\t{}\t{:.2}%",
            code,
            count,
            (*count as f64 / total_requests as f64) * 100.0
        );
    }

    println!("\nLatency Distribution");
    println!("Percentile\tLatency (ms)");
    println!("P50\t{}", median_time);
    println!("P75\t{}", times[(times.len() as f64 * 0.75) as usize]);
    println!("P95\t{}", p95_time);
    println!("P99\t{}", p99_time);
    println!("Max\t{}", max_time);

    // Additional metrics
    let min_success_time = data
        .iter()
        .filter(|d| d.status.is_success())
        .map(|d| d.time)
        .min()
        .unwrap_or(0);
    let max_success_time = data
        .iter()
        .filter(|d| d.status.is_success())
        .map(|d| d.time)
        .max()
        .unwrap_or(0);
    let avg_success_time: f64 = data
        .iter()
        .filter(|d| d.status.is_success())
        .map(|d| d.time)
        .sum::<u64>() as f64
        / successful_requests as f64;

    println!("\nAdditional Metrics");
    println!("Min Successful Request Time: {} ms", min_success_time);
    println!("Max Successful Request Time: {} ms", max_success_time);
    println!("Avg Successful Request Time: {:.2} ms", avg_success_time);
}

/// Function to save the results to a file
fn save_results(data: &[ResponseDetails], output: &str) {
    let mut file = File::create(output).expect("Unable to create file");
    for detail in data {
        writeln!(file, "{:?}", detail).expect("Unable to write data");
    }
}
