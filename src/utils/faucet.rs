use std::collections::HashMap;
use crate::utils::config::{Config};
use reqwest::{Client};
use serde_json::{Value};
use log::{info};
use rand::prelude::SliceRandom;
use rand::Rng;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use crate::utils::captcha_solver::{invisible_recaptchav2_rucaptcha};

fn generate_user_agent() -> String {
    let browsers = vec![
        ("Chrome", rand::thread_rng().gen_range(100..115)),
        ("Firefox", rand::thread_rng().gen_range(100..115)),
        ("Safari", rand::thread_rng().gen_range(10..15)),
        ("Opera", rand::thread_rng().gen_range(70..81)),
        ("Edge", rand::thread_rng().gen_range(80..91))
    ];

    let platforms = vec![
        "Windows NT 10.0",
        "Macintosh; Intel Mac OS X 10_14_6",
        "X11; Linux x86_64"
    ];

    let platform = platforms.choose(&mut rand::thread_rng()).unwrap();
    let (browser, version) = browsers.choose(&mut rand::thread_rng()).unwrap();

    let gecko = if *browser == "Firefox" {
        "Gecko/20100101"
    } else {
        ""
    };

    let chrome_version = rand::thread_rng().gen_range(80..101);
    let user_agent = format!(
        "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.{} {} {}/{}",
        platform, chrome_version, rand::thread_rng().gen_range(0..1000), gecko, browser, version
    );

    user_agent
}

fn generate_headers() -> HeaderMap {
        let ua = generate_user_agent();
        let headers = vec![
            ("authority", "faucets-backend.eu-central-1.bwarelabs.app"),
            ("accept", "application/json"),
            ("accept-language", "ru-RU,ru;q=0.9,en-US;q=0.8,en;q=0.7"),
            ("content-type", "application/json"),
            ("origin", "https://bwarelabs.com"),
            ("referer", "https://bwarelabs.com/"),
            ("sec-ch-ua", ""),
            ("sec-ch-ua-mobile", "?0"),
            ("sec-ch-ua-platform", "\"Windows\""),
            ("sec-fetch-dest", "empty"),
            ("sec-fetch-mode", "cors"),
            ("sec-fetch-site", "same-site"),
            ("user-agent", &ua),
        ];
        headers.into_iter()
            .filter_map(|(k, v)| {
                let key = HeaderName::from_bytes(k.as_bytes()).ok()?;
                let value = HeaderValue::from_str(v).ok()?;
                Some((key, value))
            })
            .collect()
    }

pub async fn bwarelabs_faucet (
        session: &Client,
        address: &str,
        config: &Config,
    ) -> Result<(), String> {
    let headers = generate_headers();

    let cap_key = &config.settings.cap_key;
    let website_url = "https://bwarelabs.com/faucets/scroll-testnet";
    let website_key = "6LcJU64nAAAAAAth2cBz5--UdVzf06B_8kNfv-JS";
    let mut captcha = String::new();
    let result = invisible_recaptchav2_rucaptcha(session, website_url, website_key, &cap_key).await;
    match result {
        Ok(cap) => {
            captcha = cap;
        },
        Err(e) => return Err(format!("Error solving captcha: {}", e)),
    }

    let mut data = HashMap::new();
    data.insert("captchaCode".to_string(), &captcha);

    let url = format!("https://faucets-backend.eu-central-1.bwarelabs.app/transfer/SCROLL_SEPOLIA/{}", address);
    let response = session
        .post(url)
        .headers(headers.clone())
        .json(&data)
        .send()
        .await;

    match response {
    Ok(res) => {
        let text = res.text().await.unwrap_or_default();
        let parsed: Value = serde_json::from_str(&text).unwrap_or_default();

        if let Some(tx_id) = parsed["txId"].as_str() {
            info!("| {} | Your daily ETH token has been successfully transferred https://sepolia-blockscout.scroll.io/tx/{}", address, tx_id);
            Ok(())
        } else if let Some(error) = parsed["error"].as_object() {
            if let Some(message) = error["message"].as_str() {
                Err(message.to_string())
            } else {
                Err("Unknown error message".to_string())
            }
        } else {
            Err(format!("Unexpected response format: {}", text))
        }
    },
    Err(e) => {
        Err(format!("Error sending request: {}", e))
    }
}
}