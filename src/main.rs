use clap::Parser;
use colored::Colorize;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::from_str;
use std::error::Error;
use tokio;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: TrackingData,
}

#[derive(Debug, Deserialize)]
struct TrackingData {
    checkpoints: Vec<Checkpoint>,
}

#[derive(Debug, Deserialize)]
struct Checkpoint {
    time: String,
    status_raw: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg()]
    tracking_number: String,

    #[arg(short, long)]
    provider: String,
}

async fn get_tracker_info(
    tracking_number: &str,
    provider: &str,
) -> Result<ApiResponse, Box<dyn Error>> {
    let url = format!(
        "https://gdeposylka.ru/api/v4/tracker/{}/{}",
        provider, tracking_number
    );

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        "X-Authorization-Token",
        HeaderValue::from_static(
            "e1e9872ba84c0e91a99bf560f92bf60b572cb03074497d59021c3f5904494f6103cfd9b227c4ed9e",
        ),
    );

    let client = reqwest::Client::new();
    let response = client.get(&url).headers(headers).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await?;
        eprintln!("Error: {} - Response Body: {}", status, body);
        return Err(format!("Request failed: {}", status).into());
    }

    let response_body = response.text().await?;
    let api_response: ApiResponse = from_str(&response_body).map_err(|e| {
        eprintln!("Failed to decode response body: {}", e);
        e
    })?;

    Ok(api_response)
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match get_tracker_info(&args.tracking_number, &args.provider).await {
        Ok(api_response) => {
            for checkpoint in api_response.data.checkpoints {
                let status = if checkpoint.status_raw == "GTMS_SIGNED" {
                    "Received".to_string()
                } else {
                    checkpoint.status_raw.clone()
                };
                println!("[{}] {}", checkpoint.time, status.bold().cyan());
            }
        }
        Err(e) => eprintln!("Error fetching tracking information: {}", e),
    }
}
