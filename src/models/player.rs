use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub role_id: Option<Uuid>,
    pub is_ready: bool,
}

impl Player {
    pub fn new(user_id: Uuid, username: String) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            user_id,
            username,
            role_id: None,
            is_ready: false,
        }
    }

    pub fn assign_role(&mut self, role_id: Uuid) {
        self.role_id = Some(role_id);
    }

    pub fn toggle_ready(&mut self) {
        self.is_ready = !self.is_ready;
    }
}