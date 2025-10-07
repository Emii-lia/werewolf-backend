use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateGuestRequest {
    pub username: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GuestSessionResponse {
    pub session_id: String,
    pub username: String,
    pub token: String,
}
