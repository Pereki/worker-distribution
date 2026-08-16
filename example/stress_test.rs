use core::num;
use std::sync::Arc;

use clap::{Parser, ValueEnum};
use reqwest::{Client, Response};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ExampleArguments {
    /// The url that should be stresstesteed
    #[arg(short, long)]
    pub url: String,

    /// Network port to use
    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1..))]
    pub amount: u16,
}

#[tokio::main]
async fn main() {
    let args = ExampleArguments::parse();
    let url = Arc::new(args.url.clone());

    let mut tasks = Vec::new();
    println!("Sending {} request to {}", args.amount, url);
    for i in 0..args.amount.clone() {
        tasks.push(tokio::spawn(query(url.clone(), i)));
    }

    for task in tasks {
        let resp = task.await;
        let resp_json = resp
            .unwrap()
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        println!("{}", resp_json)
    }
}

async fn query(url: Arc<String>, number: u16) -> Option<Response> {
    let client = Client::new();
    let body = serde_json::json!({
        "code": format!("echo \"Test {}\"", number),
        "lang": "SHELL",
    });
    println!("Sending reequest {}...", number);
    client.post(url.as_str()).json(&body).send().await.ok()
}
