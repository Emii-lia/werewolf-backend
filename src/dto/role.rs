use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::models::RoleType;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub image: Option<String>,
    pub role_type: RoleType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RoleCreateRequest {
    pub name: String,
    pub description: String,
    pub image: Option<String>,
    pub role_type: RoleType,
}

#[derive(Debug, Clone, Serialize, Deserialize,ToSchema)]
pub struct RoleUpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub role_type: Option<RoleType>,
}