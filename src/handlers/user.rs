use std::collections::HashMap;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum::extract::Query;
use redis::AsyncCommands;
use uuid::Uuid;
use crate::{
    state::AppState,
    dto::UserResponse,
    models::User,
    middleware::AuthUser,
};
use crate::dto::VerifyUsernameResponse;

#[utoipa::path(
    get,
    path = "/api/users/search",
    params(
        ("username" = String, Query, description = "Username search query (case insensitive, partial match)")
    ),
    responses(
        (status = 200, description = "Users found", body = Vec<UserResponse>)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn get_user_by_username(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _auth: AuthUser,
) -> Result<Json<Vec<UserResponse>>, (StatusCode, String)> {
    let search_query = params.get("username")
        .ok_or((StatusCode::BAD_REQUEST, "Username query not provided".to_string()))?
        .to_lowercase();

    let mut conn = state.redis.clone();
    let keys: Vec<String> = conn.keys("user:*")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut user_responses = Vec::new();
    for user_key in keys {
        let user_data: String = conn.get(&user_key).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let user: User = serde_json::from_str(&user_data)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if user.username.to_lowercase().contains(&search_query) {
            user_responses.push(UserResponse {
                id: user.id,
                username: user.username,
            });
        }
    }

    Ok(Json(user_responses))
}

#[utoipa::path(
    get,
    path = "/api/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID to fetch")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<Json<UserResponse>, (StatusCode, String)>{
    let mut conn = state.redis.clone();
    let key = format!("user:{}", id);

    let user_data: Option<String> = conn.get(&key).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user_data = user_data
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let user: User = serde_json::from_str(&user_data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
    }))
}

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of all users", body = Vec<UserResponse>)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn get_users(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<Vec<UserResponse>>, (StatusCode, String)> {
    let mut conn = state.redis.clone();
    let users: Vec<String> = conn.keys("user:*")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut user_responses = Vec::new();
    for user_key in users {
        let user_data: String = conn.get(&user_key).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let user: User = serde_json::from_str(&user_data)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        user_responses.push(UserResponse {
            id: user.id,
            username: user.username,
        });
    }

    Ok(Json(user_responses))
}

#[utoipa::path(
    get,
    path = "/api/users/verify/{username}",
    params(
        ("username" = String, Path, description = "Username to verify existence")
    ),
    responses(
        (status = 200, description = "Username existence verified", body = VerifyUsernameResponse)
    ),
    tag = "users"
)]
pub async fn verify_username_exists(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<VerifyUsernameResponse>, (StatusCode, String)> {
    let mut conn = state.redis.clone();
    let keys: Vec<String> = conn.keys("user:*")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let username = username.to_lowercase();

    for key in keys {
        let user_data: String = conn.get(&key).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let user: User = serde_json::from_str(&user_data)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if user.username.to_lowercase() == username {
            return Ok(Json(VerifyUsernameResponse {
                exists: true,
            }));
        }
    }

    Ok(Json(VerifyUsernameResponse {
        exists: false,
    }))
}