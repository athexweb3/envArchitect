use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use bb8_redis::redis::Script;

// Lua Script for Atomic Token Bucket
// Keys: [rate_limit_key]
// Args: [capacity, refill_rate_per_sec, requested_tokens, now_ts_seconds]
const LUA_SCRIPT: &str = r#"
local key = KEYS[1]
local capacity = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local requested = tonumber(ARGV[3])
local now = tonumber(ARGV[4])

local last_ts = tonumber(redis.call('HGET', key, 'ts') or now)
local tokens = tonumber(redis.call('HGET', key, 'tokens') or capacity)

-- Refill tokens based on time elapsed
local delta = math.max(0, now - last_ts)
local filled_tokens = math.min(capacity, tokens + (delta * refill_rate))

if filled_tokens >= requested then
    -- Consumption allowed
    local new_tokens = filled_tokens - requested
    redis.call('HSET', key, 'ts', now, 'tokens', new_tokens)
    redis.call('EXPIRE', key, 60) -- Verify TTL
    return {1, new_tokens}
else
    -- Rejected
    return {0, filled_tokens}
end
"#;

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    // 1. Identify Client
    // Priority: API Key (Header) > IP Address
    let (key, limit, rate) = identify_client(&req);

    // 2. Redis Check via Lua Script
    let allowed = check_rate_limit(&state, &key, limit, rate).await;

    match allowed {
        Ok(true) => next.run(req).await,
        Ok(false) => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response(),
        Err(e) => {
            tracing::error!("Rate limit redis error: {}", e);
            // Fail open? or Fail closed?
            // For V1 Security, let's Fail Open to avoid downtime if Redis blips,
            // BUT log heavily.
            next.run(req).await
        }
    }
}

fn identify_client(req: &Request<Body>) -> (String, u32, u32) {
    // Check Authorization Header for "env_"
    if let Some(auth) = req.headers().get("authorization") {
        if let Ok(val) = auth.to_str() {
            if val.starts_with("Bearer env_") {
                // High Tier: 5000 req/min (~83/sec)
                // Key: "rl:key:{hash_of_token?}"
                // We should probably hash it to avoid storing raw keys in Redis keys if possible,
                // but for rate limiting, the prefix is mostly public.
                // Let's use the full token as key for now (epheremal).
                return (format!("rl:key:{}", val), 5000, 83);
            }
        }
    }

    // Fallback: IP Address
    // We assume behind proxy, so X-Forwarded-For?
    // Axum `ConnectInfo` is tricky behind load balancers without correct config.
    // For MVP, we stick to a placeholder or simple extraction.
    // Real implementation would use `axum_client_ip`.
    let ip = "unknown_ip"; // TODO: Extract real IP

    // Low Tier: 60 req/min (1/sec)
    (format!("rl:ip:{}", ip), 60, 1)
}

async fn check_rate_limit(
    state: &AppState,
    key: &str,
    capacity: u32,
    rate: u32,
) -> anyhow::Result<bool> {
    let mut conn = state.redis.get().await?;
    let script = Script::new(LUA_SCRIPT);

    let now = chrono::Utc::now().timestamp();

    let result: Vec<i64> = script
        .key(key)
        .arg(capacity)
        .arg(rate)
        .arg(1) // requested
        .arg(now)
        .invoke_async(&mut *conn)
        .await?;

    Ok(result[0] == 1)
}
