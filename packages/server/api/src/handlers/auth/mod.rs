use axum::{routing::get, Router};

pub mod device;
pub mod github;
pub mod keys;
pub mod portal;
pub mod session;
pub mod token;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", get(github::login_handler))
        .route("/auth/callback", get(github::callback_handler))
        .route("/oauth/device/code", get(device::initiate_handler))
        .route("/oauth/token", get(token::poll_handler))
        .route("/auth/verify", get(device::verify_handler))
        .route("/auth/me", get(token::me_handler))
        .route("/auth/portal", get(portal::portal_handler))
        .route("/auth/session", get(session::session_handler))
        .route("/auth/logout", get(session::logout_handler))
        .route("/auth/keys", get(keys::list_keys).post(keys::create_key))
        .route("/auth/keys/:id", axum::routing::delete(keys::revoke_key))
        .route(
            "/auth/register-key",
            axum::routing::post(keys::register_signing_key),
        )
        .route(
            "/auth/ghcr-token",
            get(token::ghcr_token_handler).put(token::store_ghcr_pat_handler),
        )
}
