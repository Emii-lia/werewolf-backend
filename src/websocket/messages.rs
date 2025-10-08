use crate::dto::{PlayerDetails, PlayerWithRole};
use crate::models::GameState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum ClientMessage {
    #[serde(rename_all = "snake_case")]
    CreateRoom { room_name: String },
    #[serde(rename_all = "snake_case")]
    JoinRoom { room_id: Uuid },
    #[serde(rename_all = "snake_case")]
    LeaveRoom { room_id: Uuid },
    #[serde(rename_all = "snake_case")]
    GetRoomState { room_id: Uuid },
    #[serde(rename_all = "snake_case")]
    ToggleReady { room_id: Uuid },
    #[serde(rename_all = "snake_case")]
    SendMessage { room_id: Uuid, message: String },
    #[serde(rename_all = "snake_case")]
    StartGame { room_id: Uuid },
    #[serde(rename_all = "snake_case")]
    RemovePlayer { room_id: Uuid, user_id: Uuid },
    #[serde(rename_all = "snake_case")]
    ReassignRoles { room_id: Uuid  },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum ServerMessage {
    #[serde(rename_all = "snake_case")]
    RoomCreated { room_id: Uuid, room_name: String },
    #[serde(rename_all = "snake_case")]
    RoomJoined {
        room_id: Uuid,
        players: Vec<PlayerDetails>,
        game_state: GameState,
        room_name: String,
        host_id: Uuid,
        max_players: usize,
    },
    #[serde(rename_all = "snake_case")]
    RoomLeft { room_id: Uuid },
    #[serde(rename_all = "snake_case")]
    PlayerJoined {
        room_id: Uuid,
        player: PlayerDetails,
    },
    #[serde(rename_all = "snake_case")]
    PlayerLeft { room_id: Uuid, user_id: Uuid },
    #[serde(rename_all = "snake_case")]
    PlayerReady {
        room_id: Uuid,
        user_id: Uuid,
        is_ready: bool,
    },
    #[serde(rename_all = "snake_case")]
    PlayerKicked { room_id: Uuid, user_id: Uuid },
    #[serde(rename_all = "snake_case")]
    Message {
        room_id: Uuid,
        user_id: Uuid,
        username: String,
        message: String,
    },
    #[serde(rename_all = "snake_case")]
    GameStarting { room_id: Uuid },
    #[serde(rename_all = "snake_case")]
    RoleAssigned { role_id: Uuid },
    #[serde(rename_all = "snake_case")]
    AllRolesAssigned { players: Vec<PlayerWithRole> },
    #[serde(rename_all = "snake_case")]
    Error { message: String },
}
