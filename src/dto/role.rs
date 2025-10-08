use crate::models::RoleType;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub image: Option<String>,
    pub role_type: RoleType,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RoleCreateRequest {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub image: Option<String>,
    pub role_type: RoleType,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RoleUpdateRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub role_type: Option<RoleType>,
    pub priority: Option<i32>,
}

pub struct RoleAssignment {
    pub player_id: Uuid,
    pub role_id: Uuid,
}

pub struct RoleDistribution {
    pub beast_count: usize,
    pub citizen_count: usize,
    pub special_count: usize,
}

impl RoleDistribution {
    pub fn for_players(count: usize) -> Self {
        match count {
            3..=4 => Self {
                beast_count: 1,
                citizen_count: count - 1,
                special_count: 0,
            },
            5..=6 => Self {
                beast_count: 1,
                citizen_count: count - 2,
                special_count: 1,
            },
            7..=10 => Self {
                beast_count: 2,
                citizen_count: count - 4,
                special_count: 2,
            },
            11..=14 => Self {
                beast_count: 3,
                citizen_count: count - 6,
                special_count: 3,
            },
            _ => Self {
                beast_count: count / 4,
                citizen_count: count / 2,
                special_count: count / 4,
            },
        }
    }
}

pub struct RolesByType {
    pub beasts: Vec<RoleResponse>,
    pub citizens: Vec<RoleResponse>,
    pub special: Vec<RoleResponse>,
}
