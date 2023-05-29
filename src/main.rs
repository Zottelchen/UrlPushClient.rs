use dotenv::dotenv;
use reqwest::blocking::ClientBuilder;
use reqwest::header::HeaderValue;
use reqwest::header::ACCEPT;
use std::env;
use std::io::{self, Read};
use url::Url;
use webbrowser;

fn main() {
    dotenv().ok();
    let address: &str = &env::var("UPC_ADDRESS").unwrap_or("".to_string());
    let auth_user: &str = &env::var("UPC_USERNAME").unwrap_or("".to_string());
    let auth_pw: Option<String> = env::var("UPC_PASSWORD").ok();
    let pool: &str = &env::var("UPC_POOL").unwrap_or("".to_string());
    let include_seen: bool = env::var("UPC_INCLUDE_SEEN")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap();
    if address.is_empty() {
        println!("Please set the environment variable UPC_ADDRESS");
        std::process::exit(1);
    }
    if pool.is_empty() {
        println!("Please set the environment variable UPC_POOL");
        std::process::exit(1);
    }
    if auth_user.is_empty() || auth_pw.is_none() {
        println!(
            "Please set the environment variable UPC_USERNAME and UPC_PASSWORD for basic auth"
        );
        std::process::exit(1);
    }

    let client = ClientBuilder::new()
        .build()
        .expect("Failed to create reqwest client");

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    let mut url = Url::parse(&address).expect("Invalid URL");
    url.query_pairs_mut()
        .append_pair("pool", &pool)
        .append_pair("include_seen", &include_seen.to_string());

    println!("Fetching urls from {}", url.as_str());

    let response = client
        .get(url)
        .basic_auth(auth_user, auth_pw)
        .headers(headers)
        .send()
        .expect("Failed to send request");

    println!("Opening unseen urls...");
    match response.json::<serde_json::Value>() {
        Ok(json) => {
            if let Some(webinfos) = json["unrequested"].as_array() {
                for webinfo in webinfos {
                    if let Some(url_str) = webinfo["url"].as_str() {
                        println!("Opening {} - {}", webinfo["title"], url_str);
                        webbrowser::open(url_str).expect("Failed to open url");
                    }
                }
            }

            if include_seen {
                println!("\nOpening previously seen urls...");

                if let Some(webinfos) = json["requested"].as_array() {
                    for webinfo in webinfos {
                        if let Some(url_str) = webinfo["url"].as_str() {
                            println!("Opening {} - {}", webinfo["title"], url_str);
                            webbrowser::open(url_str).expect("Failed to open url");
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    let mut buffer = [0; 1]; // Buffer to store the input

    // Read a single byte from the standard input stream
    let stdin = io::stdin();
    println!("\nPress Enter to exit...");
    stdin.lock().read_exact(&mut buffer).unwrap();
}
