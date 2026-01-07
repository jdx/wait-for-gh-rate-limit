use reqwest::header::USER_AGENT;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    let resource = env::args().skip(1).find(|arg| !arg.starts_with('-')).unwrap_or(String::from("core"));
    let resources = resp.json::<RateLimitResponse>().await?.resources;
    let rate_limit = resources.get(&resource).ok_or_else(|| format!("GitHub rate limit resource '{resource}' not found"))?;

    let reset_time = chrono::DateTime::from_timestamp(rate_limit.reset as i64, 0)
        .unwrap()
        .naive_local();
    let duration = Duration::from_secs(rate_limit.reset - chrono::Utc::now().timestamp() as u64);
    let rel_time = humantime::format_duration(duration);
    // There are no docs on it as of 2026-01-07,
    // but a request when remaining==1 would already be rejected due to exceeding the limit, at least for code_search,
    // even though one would expect that to happen only at remaining==0.
    if rate_limit.remaining <= 1 {
        eprintln!("GitHub {resource} rate limit exceeded, sleeping for {rel_time} until {reset_time}");
        sleep(duration).await;
    } else if env::args().all(|arg| arg != "--quiet" && arg != "-q") {
        let remaining = rate_limit.remaining;
        let limit = rate_limit.limit;
        println!("GitHub {resource} rate limit: {remaining}/{limit} - resets at {reset_time} (~{rel_time})");
    }
    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct RateLimitResponse {
    resources: HashMap<String, RateLimit>
}

#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
struct RateLimit {
    limit: u32,
    remaining: u32,
    reset: u64,
}
