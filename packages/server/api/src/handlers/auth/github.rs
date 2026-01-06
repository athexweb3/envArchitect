use crate::services::auth_service::AuthService;
use crate::state::AppState;
use axum::http::StatusCode;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use tower_sessions::Session;

pub async fn login_handler() -> impl IntoResponse {
    let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_else(|_| "".to_string());

    if client_id.is_empty() || client_id == "your_client_id_here" {
        return (
            StatusCode::BAD_REQUEST,
            "GitHub Client ID is not configured. Please update your .env file with real credentials."
        ).into_response();
    }

    let redirect_uri = std::env::var("GITHUB_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string());

    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email%20write:packages%20repo",
        client_id, redirect_uri
    );

    Redirect::to(&url).into_response()
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
}

#[derive(Deserialize, Debug)]
struct GithubTokenResponse {
    access_token: String,
    #[allow(dead_code)]
    pub scope: Option<String>,
}

#[derive(Deserialize)]
struct GithubUserResponse {
    id: i64,
    login: String,
    email: Option<String>,
}

pub async fn callback_handler(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<CallbackQuery>,
) -> Response {
    let client_id = std::env::var("GITHUB_CLIENT_ID").ok();
    let client_secret = std::env::var("GITHUB_CLIENT_SECRET").ok();

    if client_id.is_none() || client_secret.is_none() {
        tracing::warn!(
            "GITHUB_CLIENT_ID or GITHUB_CLIENT_SECRET not set. Falling back to mock auth."
        );
        return mock_auth(state, session).await;
    }

    let client = reqwest::Client::new();

    // 1. Exchange code for access token
    let token_res = match client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id.unwrap()),
            ("client_secret", client_secret.unwrap()),
            ("code", query.code),
        ])
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to exchange token: {}", e),
            )
                .into_response()
        }
    };

    let token_data: GithubTokenResponse = match token_res.json().await {
        Ok(t) => {
            tracing::info!("GitHub OAuth Token Response: {:?}", t);
            t
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse token response: {}", e),
            )
                .into_response()
        }
    };

    // 2. Fetch user profile
    let user_res = match client
        .get("https://api.github.com/user")
        .header(
            "Authorization",
            format!("token {}", token_data.access_token),
        )
        .header("User-Agent", "EnvArchitect-Registry")
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch user profile: {}", e),
            )
                .into_response()
        }
    };

    let github_user: GithubUserResponse = match user_res.json().await {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse user response: {}", e),
            )
                .into_response()
        }
    };

    // 3. Authenticate in DB
    let auth_service = AuthService::new(state.db.clone());
    let user = match auth_service
        .authenticate_github(
            github_user.id,
            &github_user.login,
            github_user.email.as_deref(),
            &token_data.access_token,
        )
        .await
    {
        Ok(u) => u,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 4. Set session
    if let Err(e) = session.insert("user_id", user.id).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    Redirect::to("http://localhost:3001/auth/verify?message=Login successful").into_response()
}

async fn mock_auth(state: AppState, session: Session) -> Response {
    let auth_service = AuthService::new(state.db.clone());
    let github_id = 101;
    let username = "demo_dev";
    let email = Some("demo@example.com");

    let user = match auth_service
        .authenticate_github(github_id, username, email, "mock_token")
        .await
    {
        Ok(u) => u,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let _ = session.insert("user_id", user.id).await;
    Redirect::to("http://localhost:3001/auth/verify?message=Mock Login successful").into_response()
}
