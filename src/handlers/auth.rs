use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use redis::AsyncCommands;
use uuid::Uuid;
use crate::{
    state::AppState,
    dto::{SignupRequest, LoginRequest, LoginResponse, UserResponse},
    models::User,
    utils::{hash_password, verify_password, generate_token},
};

#[utoipa::path(
    post,
    path = "/api/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Username already exists")
    ),
    tag = "auth"
)]
pub async fn signup(
    State(state): State<AppState>,
    Json(data): Json<SignupRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, String)> {
    let mut conn = state.redis.clone();

    let keys: Vec<String> = conn.keys("user:*")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for user_key in keys {
        let user_data: String = conn.get(&user_key).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let existing_user: User = serde_json::from_str(&user_data)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if existing_user.username.to_lowercase() == data.username.to_lowercase() {
            return Err((StatusCode::BAD_REQUEST, "Username already exists".to_string()));
        }
    }

    let password_hash = hash_password(&data.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user_id = Uuid::new_v4();
    let user = User {
        id: user_id,
        username: data.username,
        password: password_hash,
    };

    let json = serde_json::to_string(&user)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let key = format!("user:{}", user_id);
    let _: () = conn.set(&key, json).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response = UserResponse {
        id: user.id,
        username: user.username,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(data): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let mut conn = state.redis.clone();

    let keys: Vec<String> = conn.keys("user:*")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for user_key in keys {
        let user_data: String = conn.get(&user_key).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let user: User = serde_json::from_str(&user_data)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if user.username.to_lowercase() == data.username.to_lowercase() {
            verify_password(&data.password, &user.password)
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

            let token = generate_token(user.id, &state.config.jwt.secret, state.config.jwt.expiration_hours)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            
            let user_response = UserResponse {
                id: user.id,
                username: user.username,
            };
            
            return Ok(Json(LoginResponse { 
                token,
                user: user_response,
            }));
        }
    }

    Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))
}
