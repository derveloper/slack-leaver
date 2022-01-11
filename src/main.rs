use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use clap::Parser;
use serde::Serialize;
use serde_json;
use serde_json::Value;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Actually leave
    #[clap(short, long)]
    leave: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    let token = rpassword::read_password_from_tty(Some("Token (xoxp-...): ")).unwrap();

    let _ = test_auth(token.as_str()).await?;

    let resp = list_channels(token.as_str()).await?;
    for channel in resp {
        if args.leave {
            println!("leaving #{}", channel.1);
            let _ = leave_channel(token.as_str(), channel.0).await?;
        } else {
            println!("would leave #{}", channel.1);
        }
    }
    if !args.leave {
        println!("use --leave/-l to actually leave channels")
    }
    Ok(())
}

async fn leave_channel(
    token: &str,
    channel: String,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    slack_post(token, &[("channel", channel)], "conversations.leave").await
}

async fn test_auth(token: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    slack_post(token, &[("test", "auth")], "auth.test").await
}

async fn list_channels(token: &str) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut next_cursor: Option<String>;
    let mut channels: Vec<(String, String)> = vec![];
    let mut params: HashMap<String, String> = HashMap::new();
    params.insert("limit".to_string(), "1000".to_string());

    print!("Fetching channels");
    io::stdout().flush().unwrap();

    while {
        let resp = slack_post(token, &params, "conversations.list").await?;

        let nc = resp["response_metadata"]["next_cursor"]
            .as_str()
            .map(|n| if n == "" { None } else { Some(n) })
            .flatten();
        if let Some(_next_cursor) = nc {
            next_cursor = Some(_next_cursor.to_string());
            params.insert("cursor".to_string(), _next_cursor.to_string());
        } else {
            next_cursor = None
        }
        let chan_resp = resp["channels"].as_array();
        if let Some(next_channels) = chan_resp {
            let next_channels = next_channels
                .iter()
                .filter(|c| {
                    let is_channel = c["is_channel"].as_bool().unwrap_or(false);
                    let is_member = c["is_member"].as_bool().unwrap_or(false);
                    let name = c["name"].as_str().unwrap_or("unknown");
                    is_channel && is_member && name != "general"
                })
                .map(|c| {
                    (
                        c["id"].as_str().unwrap().to_string(),
                        c["name"].as_str().unwrap().to_string(),
                    )
                })
                .collect::<Vec<(String, String)>>();
            channels = [channels, next_channels].concat();
        } else {
            next_cursor = None;
        }

        print!(".");
        io::stdout().flush().unwrap();
        sleep(Duration::from_secs(2));
        next_cursor.is_some()
    } {}

    println!(" done.");
    Ok(channels)
}

async fn slack_post<T: Serialize + ?Sized>(
    token: &str,
    data: &T,
    method: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let url = format!("{}{}", "https://slack.com/api/", method);
    let resp = reqwest::Client::new()
        .post(url)
        .bearer_auth(token)
        .form(&data)
        .send()
        .await?;

    let slack_json: Value = resp.json().await?;
    if slack_json["ok"].as_bool().unwrap_or(false) {
        Ok(slack_json)
    } else {
        Err(slack_json["error"].as_str().unwrap_or("unknown").into())
    }
}
