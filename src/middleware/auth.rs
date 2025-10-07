use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use redis::AsyncCommands;
use uuid::Uuid;
use crate::{state::AppState, utils::validate_token};
use crate::models::GuestSession;

pub async fn inject_state_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    request.extensions_mut().insert(state);
    next.run(request).await
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user_id = validate_token(token, &state.config.jwt.secret)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(user_id);

    Ok(next.run(request).await)
}

pub struct AuthUser(pub Uuid);

impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            if let Some(user_id) = parts.extensions.get::<Uuid>().copied() {
                return Ok(AuthUser(user_id));
            }

            let auth_header = parts
                .headers
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            if let Some(header) = auth_header {
                if let Some(token) = header.strip_prefix("Bearer ") {
                    let app_state = parts
                        .extensions
                        .get::<crate::state::AppState>()
                        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

                    let user_id = validate_token(token, &app_state.config.jwt.secret)
                        .ok_or(StatusCode::UNAUTHORIZED)?;
                    return Ok(AuthUser(user_id));
                }
            }

            let query = parts.uri.query().unwrap_or("");
            for param in query.split('&') {
                if let Some(token) = param.strip_prefix("token=") {
                    let app_state = parts
                        .extensions
                        .get::<crate::state::AppState>()
                        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

                    let user_id = validate_token(token, &app_state.config.jwt.secret)
                        .ok_or(StatusCode::UNAUTHORIZED)?;
                    return Ok(AuthUser(user_id));
                }
            }

            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlayerIdentity {
    Registered { user_id: Uuid, username: String },
    Guest { session_id: Uuid, username: String },
}

impl PlayerIdentity {
    pub fn id(&self) -> Uuid {
        match self {
            PlayerIdentity::Registered { user_id, .. } => *user_id,
            PlayerIdentity::Guest { session_id, .. } => *session_id,
        }
    }

    pub fn username(&self) -> &str {
        match self {
            PlayerIdentity::Registered { username, .. } => username,
            PlayerIdentity::Guest { username, .. } => username,
        }
    }

    pub fn is_guest(&self) -> bool {
        matches!(self, PlayerIdentity::Guest { .. })
    }
}

pub struct Player(pub PlayerIdentity);

impl<S> axum::extract::FromRequestParts<S> for Player
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let auth_header = parts
                .headers
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            let token = if let Some(header) = auth_header {
                header.strip_prefix("Bearer ")
            } else {
                let query = parts.uri.query().unwrap_or("");
                query
                    .split('&')
                    .find_map(|param| param.strip_prefix("token="))
            };

            let token = token.ok_or(StatusCode::UNAUTHORIZED)?;

            let app_state = parts
                .extensions
                .get::<crate::state::AppState>()
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

            let id = validate_token(token, &app_state.config.jwt.secret)
                .ok_or(StatusCode::UNAUTHORIZED)?;

            let mut conn = app_state.redis.clone();

            let guest_key = format!("guest:{}", id);
            if let Ok(Some(guest_data)) = conn.get::<_, Option<String>>(&guest_key).await {
                if let Ok(guest) = serde_json::from_str::<GuestSession>(&guest_data) {
                    return Ok(Player(PlayerIdentity::Guest {
                        session_id: guest.session_id,
                        username: guest.username,
                    }));
                }
            }

            let user_key = format!("user:{}", id);
            if let Ok(Some(user_data)) = conn.get::<_, Option<String>>(&user_key).await {
                if let Ok(user) = serde_json::from_str::<crate::models::User>(&user_data) {
                    return Ok(Player(PlayerIdentity::Registered {
                        user_id: user.id,
                        username: user.username,
                    }));
                }
            }

            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
