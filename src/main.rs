use reqwest::header::USER_AGENT;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let mut req = reqwest::Client::new().get("https://api.github.com/rate_limit");
    let token = env::var("GITHUB_TOKEN")
        .or(env::var("GITHUB_API_TOKEN"))
        .ok();
    req = req.header(USER_AGENT, env!("CARGO_PKG_NAME"));
    req = req.header("x-github-api-version", "2022-11-28");
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await?;
    resp.error_for_status_ref()?;
    let rate_limit = resp.json::<RateLimitResponse>().await?.rate;
    let reset_time = chrono::DateTime::from_timestamp(rate_limit.reset as i64, 0)
        .unwrap()
        .naive_local();
    let duration = Duration::from_secs(rate_limit.reset - chrono::Utc::now().timestamp() as u64);
    let rel_time = humantime::format_duration(duration);
    if rate_limit.remaining == 0 {
        eprintln!("Rate limit exceeded, sleeping for {rel_time} until {reset_time}",);
        sleep(duration).await;
    } else if env::args().all(|arg| arg != "--quiet" && arg != "-q") {
        let remaining = rate_limit.remaining;
        let limit = rate_limit.limit;
        println!("GitHub rate limit: {remaining}/{limit} - resets at {reset_time} (~{rel_time})");
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
