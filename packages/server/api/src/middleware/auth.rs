use crate::services::auth_service::AuthService;
use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use database::models::User;
use shared::keys;
use tower_sessions::Session;

#[derive(Clone, Debug)]
pub struct AuthUser(pub User);

pub async fn auth_middleware(
    State(state): State<AppState>,
    session: Session,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let mut user: Option<User> = None;
    let auth_service = AuthService::new(state.db.clone());

    // 1. Check Session Auth
    if let Ok(Some(user_id)) = session.get::<uuid::Uuid>("user_id").await {
        if let Ok(Some(u)) = auth_service.find_user_by_id(user_id).await {
            user = Some(u);
        }
    }

    // 2. Check Bearer Token (if no session or session invalid)
    // Supports:
    // - JWT Bearer (for device flow access tokens)
    // - API Key "env_..."
    if user.is_none() {
        if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];

                    if token.starts_with("env_") {
                        // A. API Key
                        // Fast Checksum (CPU)
                        if keys::validate_key_format(token) {
                            // Slow Hash Check (DB)
                            // We need to look up the key.
                            // Optimized: We should use lookup_hash (SHA256) if we implemented it,
                            // but for now we might have to scan or assume we can't look up by raw key
                            // without the lookup_hash from the key creation.

                            // Wait, `0006_api_keys.sql` added `lookup_hash`.
                            // But my `generate_api_key` logic might not have exposed the lookup hash logic to the client?
                            // No, client sends RAW key. Server computes SHA256(raw) -> Look up row -> Verify Argon2(raw).
                            // Actually, since we have the RAW key here, we can compute the SHA256 lookup hash.
                            // BUT, `shared::keys` does not expose a "compute_lookup_hash" yet?
                            // It does expose `generate_api_key`.

                            // Let's implement `lookup_key_by_value` in AuthService or here?
                            // AuthService is better.

                            // Secure DB Check
                            if let Ok(u) = auth_service.verify_api_key(token).await {
                                user = Some(u);
                            }
                        }
                    } else {
                        // B. JWT Access Token (Device Flow)
                        if let Ok(claims) = auth_service.verify_token(token) {
                            if let Ok(uid) = uuid::Uuid::parse_str(&claims.sub) {
                                if let Ok(Some(u)) = auth_service.find_user_by_id(uid).await {
                                    user = Some(u);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Inject User into Extensions
    if let Some(u) = user {
        req.extensions_mut().insert(AuthUser(u));
    }

    next.run(req).await
}
