use crate::dto::RoleResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct PlayerDetails {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub role_id: Option<Uuid>,
    pub is_ready: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerWithRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub role_id: Option<Uuid>,
    pub is_ready: bool,
    pub role: Option<RoleResponse>,
}
