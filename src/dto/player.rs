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