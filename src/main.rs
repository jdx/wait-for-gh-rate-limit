use std::env;
use std::time::Duration;
use reqwest::header::USER_AGENT;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let mut req = reqwest::Client::new().get("https://api.github.com/rate_limit");
    let token = env::var("GITHUB_TOKEN").or(env::var("GITHUB_API_TOKEN")).ok();
    req = req.header(USER_AGENT, env!("CARGO_PKG_NAME"));
    req = req.header("x-github-api-version", "2022-11-28");
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await?;
    resp.error_for_status_ref()?;
    let rate_limit = resp.json::<RateLimitResponse>().await?.rate;
    println!("{:?}", rate_limit);
    if rate_limit.remaining == 0 {
        let duration = Duration::from_secs(rate_limit.reset - chrono::Utc::now().timestamp() as u64);
        let reset_time = chrono::DateTime::from_timestamp(rate_limit.reset as i64, 0).unwrap();
        eprintln!("Rate limit exceeded, sleeping for {} until {}", humantime::format_duration(duration), reset_time.naive_local());
        sleep(duration).await;
    }
    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct RateLimitResponse {
    rate: RateLimit,
}

#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
struct RateLimit {
    limit: u32,
    remaining: u32,
    reset: u64,
}
