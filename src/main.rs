use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::timeout;
use reqwest::Client;
use clap::{Arg, App};
use lazy_static::lazy_static;

// Limit concurrency to prevent system overload
const MAX_CONCURRENT_REQUESTS: usize = 100; // Adjust based on system performance

lazy_static! {
    static ref LIVE_FILE: Arc<Mutex<BufWriter<File>>> = Arc::new(Mutex::new(BufWriter::new(File::create("live.txt").unwrap())));
    static ref DEAD_FILE: Arc<Mutex<BufWriter<File>>> = Arc::new(Mutex::new(BufWriter::new(File::create("dead.txt").unwrap())));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Domain Checker")
        .version("1.2")
        .about("Efficiently checks if domains are active or dead")
        .arg(Arg::with_name("file")
            .short('f')
            .long("file")
            .value_name("FILE")
            .help("Sets the input file containing domains")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("verbose")
            .short('v')
            .long("verbose")
            .help("Enables verbose output"))
        .get_matches();

    let filename = matches.value_of("file").unwrap();
    let verbose = matches.is_present("verbose");

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));

    println!("Checking domains...");

    let mut handles = Vec::new();
    for line in reader.lines() {
        let domain = match line {
            Ok(d) => d.trim().to_string(),
            Err(_) => continue,
        };

        let client = client.clone();
        let verbose = verbose;
        let semaphore = Arc::clone(&semaphore);

        // Control concurrency using a semaphore
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            check_domain(&client, &domain, verbose).await;
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    println!("Domain check completed.");
    Ok(())
}

// Check a domain using both HTTP and HTTPS
async fn check_domain(client: &Client, domain: &str, verbose: bool) {
    let request_timeout = Duration::from_secs(10);

    let http_url = format!("http://{}", domain);
    let https_url = format!("https://{}", domain);

    if let Some(success) = test_url(client, &http_url, request_timeout, verbose).await {
        println!("{}", success);
        return;
    }

    if let Some(success) = test_url(client, &https_url, request_timeout, verbose).await {
        println!("{}", success);
        return;
    }

    write_to_file(&DEAD_FILE, domain).await;
    println!("{}", format_failure(domain, verbose));
}

// Test a single URL
async fn test_url(client: &Client, url: &str, timeout_duration: Duration, verbose: bool) -> Option<String> {
    match timeout(timeout_duration, client.head(url).send()).await {
        Ok(Ok(response)) if response.status().is_success() => {
            let domain = extract_domain(url);
            write_to_file(&LIVE_FILE, &domain).await;
            Some(format_success(url, response.status(), verbose))
        }
        _ => None,
    }
}

// Extract domain from URL
fn extract_domain(url: &str) -> String {
    url.replace("http://", "")
       .replace("https://", "")
       .split('/')
       .next()
       .unwrap_or(url)
       .to_string()
}

// Write to a file safely using async-friendly Mutex
async fn write_to_file(file: &Arc<Mutex<BufWriter<File>>>, domain: &str) {
    let mut file = file.lock().await;
    let _ = writeln!(file, "{}", domain);
    let _ = file.flush();
}

// Format success message
fn format_success(url: &str, status: reqwest::StatusCode, verbose: bool) -> String {
    if verbose {
        format!("✓ {} - Active (Status: {})", url, status)
    } else {
        format!("✓ {} - Active", url)
    }
}

// Format failure message
fn format_failure(domain: &str, verbose: bool) -> String {
    if verbose {
        format!("✗ {} - Failed (Tried both HTTP & HTTPS)", domain)
    } else {
        format!("✗ {} - Failed", domain)
    }
}


