use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct VerifyUsernameResponse {
    pub exists: bool,
}