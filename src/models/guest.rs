use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestSession {
    pub session_id: Uuid,
    pub username: String,
}

impl GuestSession {
    pub fn new(username: String) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            username,
        }
    }
}
