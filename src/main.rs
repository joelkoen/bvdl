use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use ureq::Agent;

const LIMIT: usize = 100;

const PASSKEY_START: &str = ",passkey:\"";
const PASSKEY_END: &str = "\",baseUrl";

#[derive(Debug, Parser)]
struct Cli {
    passkey: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let agent = Agent::new();

    let passkey = if cli.passkey.contains('/') {
        // this is a deployment id
        let js = agent
            .get(&format!(
                "https://display.ugc.bazaarvoice.com/static/{}/bvapi.js",
                cli.passkey
            ))
            .call()?
            .into_string()?;

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
    loop {
        let url = if offset >= 300_000 {
            format!("{base}&offset={}&sort=id:desc", offset - 300_000)
        } else {
            format!("{base}&offset={}&sort=id:asc", offset)
        };

        let page: ApiPage = agent.get(&url).call()?.into_json()?;

        if offset == 0 {
            eprintln!("Fetching {} products...", page.total_results);
            if page.total_results >= 600_000 {
                eprintln!("This site has more than 600 000 products. Due to API restrictions, only the first and last 300 000 will be fetched.");
            }
        }
        offset += page.results.len();

        for x in page.results {
            println!("{}", serde_json::to_string(&x)?);
        }

        if offset >= page.total_results {
            break;
        }
    }

    println!("{passkey}");

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ApiPage {
    total_results: usize,
    results: Vec<ApiItem>,
    // errors: Vec<ApiError>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ApiItem {
    id: String,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "PascalCase")]
// struct ApiError {
//     code: String,
//     message: String,
// }
