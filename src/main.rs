use anyhow::{bail, Context, Result};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::task::JoinSet;

const LIMIT: usize = 100;

const PASSKEY_START: &str = ",passkey:\"";
const PASSKEY_END: &str = "\",baseUrl";

#[derive(Debug, Parser)]
struct Cli {
    passkey: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = Client::new();

    let passkey = if cli.passkey.contains('/') {
        // this is a deployment id
        let js = client
            .get(format!(
                "https://display.ugc.bazaarvoice.com/static/{}/bvapi.js",
                cli.passkey
            ))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let start = js
            .find(PASSKEY_START)
            .context("Failed to find passkey start")?
            + PASSKEY_START.len();
        let end = js.find(PASSKEY_END).context("Failed to find passkey end")?;
        let passkey = js[start..end].to_string();
        eprintln!("Found passkey: {passkey}");
        passkey
    } else {
        cli.passkey
    };

    let base = format!("https://api.bazaarvoice.com/data/products.json?apiVersion=5.5&passkey={passkey}&limit={LIMIT}");
    let mut offset = 0;
    let mut total = 0;
    let mut in_flight = JoinSet::new();
    loop {
        if in_flight.len() > 10 {
            in_flight.join_next().await.unwrap()??;
        }

        let url = if offset >= 300_000 {
            format!("{base}&offset={}&sort=id:desc", offset - 300_000)
        } else {
            format!("{base}&offset={}&sort=id:asc", offset)
        };

        if offset == 0 {
            total = fetch(client.clone(), url, 0).await?;
            eprintln!("Fetching {total} products...");
            if total >= 600_000 {
                eprintln!("This site has more than 600 000 products. Due to API restrictions, only the first and last 300 000 will be fetched.");
            }
        } else {
            let next_offset = offset + LIMIT;
            let ignore_last = if next_offset >= total {
                next_offset - total
            } else {
                0
            };

            in_flight.spawn(fetch(client.clone(), url, ignore_last));
        }

        offset += LIMIT;
        if offset >= total {
            break;
        }
    }

    while let Some(result) = in_flight.join_next().await {
        result??;
    }

    Ok(())
}

async fn fetch(client: Client, url: String, ignore_last: usize) -> Result<usize> {
    let page: ApiPage = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if page.errors.len() > 0 {
        for err in page.errors {
            eprintln!("error: {} - {}", err.code, err.message);
        }
        bail!("Received API errors");
    }

    let max = page.results.len() - ignore_last;
    for (i, x) in page.results.iter().enumerate() {
        if i == max {
            break;
        }

        println!("{}", serde_json::to_string(&x)?);
    }

    Ok(page.total_results)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ApiPage {
    total_results: usize,
    results: Vec<ApiItem>,
    errors: Vec<ApiError>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ApiItem {
    id: String,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ApiError {
    code: String,
    message: String,
}
