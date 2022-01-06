use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use serde::Serialize;
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = rpassword::read_password_from_tty(Some("Token (xoxp-...): ")).unwrap();

    let resp = list_channels(token.as_str()).await;
    for channel in resp {
        let _ = leave_channel(token.as_str(), channel.0).await?;
        println!("leaving #{}", channel.1);
    }
    Ok(())
}

async fn leave_channel(token: &str, channel: String) -> Result<serde_json::Value, reqwest::Error> {
    slack_post(token, &[("channel", channel)], "conversations.leave").await
}

async fn list_channels(token: &str) -> Vec<(String, String)> {
    let mut next_cursor: Option<String> = None;
    let mut channels: Vec<(String, String)> = vec![];
    let mut params: HashMap<String, String> = HashMap::new();
    params.insert("limit".to_string(), "1000".to_string());

    print!("Fetching");
    io::stdout().flush().unwrap();

    while {
        let resp = slack_post(token, &params, "conversations.list").await;

        match resp {
            Ok(resp) => {
                let nc = resp["response_metadata"]["next_cursor"].as_str()
                    .map(|n| if n == "" { None } else { Some(n) }).flatten();
                if let Some(_next_cursor) = nc {
                    next_cursor = Some(_next_cursor.to_string());
                    params.insert("cursor".to_string(), _next_cursor.to_string());
                } else {
                    next_cursor = None
                }
                let chan_resp = resp["channels"].as_array();
                if let Some(next_channels) = chan_resp {
                    let next_channels = next_channels.iter()
                        .filter(|c| {
                            let is_channel = c["is_channel"].as_bool().unwrap_or(false);
                            let is_member = c["is_member"].as_bool().unwrap_or(false);
                            is_channel && is_member
                        }).map(|c| {
                        (c["id"].as_str().unwrap().to_string(), c["name"].as_str().unwrap().to_string())
                    }).collect::<Vec<(String, String)>>();
                    channels = [channels, next_channels].concat();
                    channels.dedup();
                } else {
                    next_cursor = None;
                }
            }
            Err(err) => println!("List channels failed {:?}", err)
        }

        print!(".");
        io::stdout().flush().unwrap();
        sleep(Duration::from_secs(2));
        next_cursor.is_some()
    } {}

    println!(" done.");
    channels
}

async fn slack_post<T: Serialize + ?Sized>(token: &str, data: &T, method: &str) -> Result<serde_json::Value, reqwest::Error> {
    let url = format!("{}{}", "https://slack.com/api/", method);
    let slack_json: serde_json::Value = reqwest::Client::new()
        .post(url)
        .bearer_auth(token)
        .form(data)
        .send()
        .await?
        .json()
        .await?;
    Ok(slack_json)
}
