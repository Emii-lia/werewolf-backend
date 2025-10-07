use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use redis::AsyncCommands;
use crate::dto::{CreateGuestRequest, GuestSessionResponse};
use crate::models::GuestSession;
use crate::state::AppState;
use crate::utils::generate_token;

#[utoipa::path(
    post,
    path = "/api/guest/session",
    request_body = CreateGuestRequest,
    responses(
        (status = 201, description = "Guest session created successfully", body = GuestSessionResponse),
        (status = 400, description = "Invalid username")
    ),
    tag = "guest"
)]
pub async fn create_guest_session(
    State(state): State<AppState>,
    Json(data): Json<CreateGuestRequest>,
) -> Result<(StatusCode, Json<GuestSessionResponse>), (StatusCode, String)> {
    let username = data.username.trim();

    if username.is_empty() || username.len() > 50 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username must be between 1 and 50 characters".to_string(),
        ));
    }

    let guest_session = GuestSession::new(username.to_string());
    let mut conn = state.redis.clone();

    let session_key = format!("guest:{}", guest_session.session_id);
    let session_json = serde_json::to_string(&guest_session)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _: () = conn
        .set_ex(&session_key, session_json, 86400)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let token = generate_token(
        guest_session.session_id,
        &state.config.jwt.secret,
        state.config.jwt.expiration_hours,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(GuestSessionResponse {
            session_id: guest_session.session_id.to_string(),
            username: guest_session.username,
            token,
        }),
    ))
}
