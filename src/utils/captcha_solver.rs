use crate::constants::*;

use reqwest::{Client};
use tokio::time::sleep;
use serde_json::{json, Value};
use std::time::{Duration};
use log::{info, error};
use crate::utils::error::MyError;


pub async fn invisible_recaptchav2_rucaptcha(
        session: &Client,
        website_url: &str,
        website_key: &str,
        cap_key: &str,
    ) -> Result<String, MyError> {

    let url = format! ("http://rucaptcha.com/in.php?key={}&method=userrecaptcha&json=1&googlekey={}&pageurl={}&invisible=1", cap_key, website_key, website_url);

    let response = session
        .post(url)
        .send()
        .await?;

    let response_data: Value = response.json().await?;
    // println!("|  | Response_data: {}", response_data);
    info!("| | Captcha - Solve...");
    if let Some(task_id) = response_data["request"].as_str() {
        // println!("task_id: {}", task_id);
        sleep(Duration::from_secs(5)).await;

        for _ in 0..MAX_RETRIES {
            let url_result = format!("http://rucaptcha.com/res.php?key={}&action=get2&id={}&json=1", cap_key, task_id);
            let response = session
                .get(url_result)
                .send()
                .await?;
            let task_result: Value = response.json().await?;
            if let Some(status) = task_result["status"].as_u64() {
                if status == 1 {
                    // println!("| | Status ready: {}", task_result);
                    if let Some(g_recaptcha_response) = task_result["request"].as_str() {
                        info!("| | Captcha - Ok");
                        // println!("| | g_recaptcha_response ready: {}", g_recaptcha_response);
                        return Ok(g_recaptcha_response.to_string());
                    }
                } else if status == 0 {
                    // println!("Status processing: {}", task_result);
                    sleep(Duration::from_secs(3)).await;
                    continue;
                } else {
                    error!("| | Captcha - Error");
                    sleep(Duration::from_secs(3)).await;
                    continue;
                }
            }
            // else {
            //     println!("No status: {}", task_result);
            //     sleep(Duration::from_secs(3)).await;
            //     continue
            // }
        }
    }

    Err(MyError::ErrorStr("Failed to solve the captcha.".to_string()))
}