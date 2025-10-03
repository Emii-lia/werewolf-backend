use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use redis::AsyncTypedCommands;
use uuid::Uuid;
use crate::dto::{RoleCreateRequest, RoleResponse, RoleUpdateRequest};
use crate::middleware::AuthUser;
use crate::models::Role;
use crate::state::AppState;

#[utoipa::path(
    post,
    path = "/api/roles",
    request_body = RoleCreateRequest,
    responses(
        (status = 201, description = "Role created successfully", body = RoleResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "roles"
)]
#[axum::debug_handler]
pub async fn create_role(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(data): Json<RoleCreateRequest>,
) -> Result<(StatusCode, Json<RoleResponse>), (StatusCode, String)> {
    let mut conn = state.redis.clone();
    
    let role = RoleResponse {
        id: uuid::Uuid::new_v4(),
        name: data.name,
        description: data.description,
        image: data.image,
        role_type: data.role_type,
    };
    
    let json = serde_json::to_string(&role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let key = format!("role:{}", role.id);
    let _: () = conn.set(&key, json).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok((StatusCode::CREATED, Json(role))) 
}

#[utoipa::path(
    get,
    path = "/api/roles",
    responses(
        (status = 200, description = "List of roles", body = Vec<RoleResponse>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "roles"
)]
pub async fn get_roles(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<Vec<RoleResponse>>, (StatusCode, String)> {
    let mut conn = state.redis.clone();
    let roles: Vec<String> = conn.keys("role:*")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let mut role_responses = Vec::new();
    for role_key in roles {
        let role_data: Option<String> = conn.get(&role_key).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let role_data = role_data
            .ok_or((StatusCode::NOT_FOUND, "Role not found".to_string()))?;
        let role: Role = serde_json::from_str(&role_data)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
    role_responses.push(RoleResponse {
            id: role.id,
            name: role.name,
            description: role.description,
            image: role.image,
            role_type: role.role_type,
        });
    }
    Ok(Json(role_responses))
}

#[utoipa::path(
    get,
    path = "/api/roles/{id}",
    params(
        ("id" = Uuid, Path, description = "Role ID to fetch")
    ),
    responses(
        (status = 200, description = "Role details", body = RoleResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Role not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "roles"
)]
pub async fn get_role_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: AuthUser
) -> Result<Json<RoleResponse>, (StatusCode, String)> {
    let mut conn = state.redis.clone();
    let key = format!("role:{}", id);

    let role_data: Option<String> = conn.get(&key).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let role_data = role_data
        .ok_or((StatusCode::NOT_FOUND, "Role not found".to_string()))?;

    let role: Role = serde_json::from_str(&role_data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(RoleResponse {
        id: role.id,
        name: role.name,
        description: role.description,
        image: role.image,
        role_type: role.role_type,
    }))
}

#[utoipa::path(
    put,
    path = "/api/roles/{id}",
    params(
        ("id" = Uuid, Path, description = "Role ID to update")
    ),
    request_body = RoleUpdateRequest,
    responses(
        (status = 200, description = "Role updated successfully", body = RoleResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Role not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "roles"
)]
pub async fn update_role(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
    Json(data): Json<RoleUpdateRequest>,
) -> Result<Json<RoleResponse>, (StatusCode, String)> {
    let mut conn = state.redis.clone();
    let key = format!("role:{}", id);
    let role_data: Option<String> = conn.get(&key).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let role_data = role_data
        .ok_or((StatusCode::NOT_FOUND, "Role not found".to_string()))?;

    let role: Role = serde_json::from_str(&role_data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let new_role = Role {
        id: role.id,
        name: data.name.unwrap_or(role.name),
        description: data.description.unwrap_or(role.description),
        image: data.image.or(role.image),
        role_type: data.role_type.unwrap_or(role.role_type),
    };

    let json = serde_json::to_string(&new_role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let _: () = conn.set(&key, json).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(RoleResponse {
        id: new_role.id,
        name: new_role.name,
        description: new_role.description,
        image: new_role.image,
        role_type: new_role.role_type,
    }))
}