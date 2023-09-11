use std::{sync::Arc, time::Duration};
use log::error;
use rand::Rng;
use reqwest::{Client, Proxy};
use tokio::sync::Semaphore;
mod utils;
mod constants;
use utils::{config, scroll, error::MyError};

async fn build_client(ip: &str, port: &str, login: &str, pass: &str) -> Result<Client, MyError> {
    let proxy = Proxy::https(format!("http://{}:{}", ip, port))?
        .basic_auth(login, pass);
    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(30))
        .build()?;
    Ok(client)
}

async fn random_delay(range: (u64, u64)) {
    let (min, max) = range;
    let delay_duration = rand::thread_rng().gen_range(min..=max);
    tokio::time::sleep(tokio::time::Duration::from_secs(delay_duration)).await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the logger
    utils::logger::setup_logger().unwrap();


    // Read config
    let arc_config = Arc::new(config::read_config("Config/Config.toml").expect("Failed to read config"));

    // Read files
    let proxy_lines = std::fs::read_to_string("FILEs/proxy.txt")?;
    let wallet_data_lines = std::fs::read_to_string("FILEs/address_private_key.txt")?;

    let paired_data: Vec<_> = proxy_lines.lines().zip(wallet_data_lines.lines()).collect();

    let max_concurrent_tasks = arc_config.threads.number_of_threads;  // Adjusted

    let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks as usize));

    let futures: Vec<_> = paired_data.into_iter().enumerate().map(|(index, (proxy_line, wallet_data_line))| {
        let wallet_data_line = wallet_data_line.to_owned();
        let proxy_parts: Vec<String> = proxy_line.split(":").map(|s| s.to_string()).collect();

        let (ip, port, login, pass) = (proxy_parts[0].clone(), proxy_parts[1].clone(), proxy_parts[2].clone(), proxy_parts[3].clone());

        let wallet_parts: Vec<&str> = wallet_data_line.split(":").collect();
        let address = wallet_parts[0].to_string();
        let private_key = wallet_parts[1].to_string();

        let sema_clone = semaphore.clone();
        let config_clone = arc_config.clone();

        tokio::spawn(async move {
            if index > 0 {
                random_delay(config_clone.threads.delay_between_threads).await;  // Add this at the beginning of the thread
            }


            // Acquire semaphore permit
            let _permit = sema_clone.acquire().await;

            let client = match build_client(&ip, &port, &login, &pass).await {
                Ok(c) => c,
                Err(e) => {
                    error!("| | Failed to build client: {}", e.to_string());
                    return;
                }
            };

            let private_key_str = if private_key.starts_with("0x") {
                &private_key[2..]
            } else {
                &private_key
            };

            scroll::execute_blockchain_operations(&private_key_str, &address, client.clone(), &config_clone).await;

        })
    }).collect();

    futures::future::join_all(futures).await;

    Ok(())
}