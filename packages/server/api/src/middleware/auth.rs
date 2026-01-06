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

    if let Ok(Some(user_id)) = session.get::<uuid::Uuid>("user_id").await {
        if let Ok(Some(u)) = auth_service.find_user_by_id(user_id).await {
            user = Some(u);
        }
    }

    if user.is_none() {
        if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];

                    if token.starts_with("env_") {
                        if keys::validate_key_format(token) {
                            // Secure DB Check
                            if let Ok(u) = auth_service.verify_api_key(token).await {
                                user = Some(u);
                            }
                        }
                    } else {
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

    if let Some(u) = user {
        req.extensions_mut().insert(AuthUser(u));
    }

    next.run(req).await
}
