use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::dto::player::PlayerDetails;
use crate::models::GameState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateRoomRequest {
    pub name: String,
    pub max_players: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RoomInfo {
    pub id: Uuid,
    pub name: String,
    pub host_id: Uuid,
    pub player_count: usize,
    pub max_players: usize,
    pub game_state: GameState,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RoomDetails {
    pub id: Uuid,
    pub name: String,
    pub host_id: Uuid,
    pub players: Vec<PlayerDetails>,
    pub max_players: usize,
    pub game_state: GameState,
}