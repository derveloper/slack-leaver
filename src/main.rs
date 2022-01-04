use serde::Serialize;
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = rpassword::read_password_from_tty(Some("Token (xoxp-...): ")).unwrap();

    let resp = list_channels(token.as_str()).await?;
    let resp = resp["channels"].as_array().unwrap()
        .iter()
        .filter(|c| {
            c["is_channel"].as_bool().unwrap()
        })
        .flat_map(|c| {
            c["id"].as_str()
        })
        .collect::<Vec<&str>>();
    println!("{:#?}", resp);

    for channel in resp {
        let _ = leave_channel(token.as_str(), channel).await?;
        println!("leaving {}", channel);
    }
    Ok(())
}

async fn leave_channel(token: &str, channel: &str) -> Result<serde_json::Value, reqwest::Error> {
    slack_post(token, &[("channel", channel)], "conversations.leave").await
}

async fn list_channels(token: &str) -> Result<serde_json::Value, reqwest::Error> {
    slack_post(token,&[("1", 0)], "conversations.list").await
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
