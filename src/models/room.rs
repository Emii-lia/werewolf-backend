use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::models::Player;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema,PartialEq)]
pub enum GameState {
    Waiting,
    Starting,
    InProgress,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub host_id: Uuid,
    pub players: Vec<Player>,
    pub max_players: usize,
    pub game_state: GameState,
}

impl Room {
    pub fn new(id: Uuid, name: String, host_id: Uuid) -> Self {
        Self {
            id,
            name,
            host_id,
            players: Vec::new(),
            max_players: 10,
            game_state: GameState::Waiting,
        }
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), String> {
        if self.players.len() >= self.max_players {
            return Err("Room is full".to_string());
        }
        if self.game_state != GameState::Waiting {
            return Err("Game already started".to_string());
        }
        self.players.push(player);
        Ok(())
    }

    pub fn remove_player(&mut self, user_id: &Uuid) {
        self.players.retain(|p| p.user_id != *user_id);
    }

    pub fn get_player_mut(&mut self, user_id: &Uuid) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.user_id == *user_id)
    }

    pub fn start_game(&mut self) -> Result<(), String> {
        if self.players.len() < 3 {
            return Err("Not enough players".to_string());
        }
        if !self.players.iter().all(|p| p.is_ready) {
            return Err("Not all players are ready".to_string());
        }
        self.game_state = GameState::Starting;
        Ok(())
    }

    pub fn assign_roles(&mut self, role_ids: Vec<Uuid>) -> Result<(), String> {
        if role_ids.len() != self.players.len() {
            return Err("Role count mismatch".to_string());
        }
        for (player, role_id) in self.players.iter_mut().zip(role_ids.iter()) {
            player.assign_role(*role_id);
        }
        self.game_state = GameState::InProgress;
        Ok(())
    }
}