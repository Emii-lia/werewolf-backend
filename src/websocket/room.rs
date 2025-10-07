use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use axum::extract::ws::Message;
use crate::models::{Player, GameState};

pub type Tx = mpsc::UnboundedSender<Message>;

#[derive(Debug, Clone)]
pub struct RoomState {
    pub rooms: Arc<RwLock<HashMap<Uuid, GameRoom>>>,
}

impl RoomState {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameRoom {
    pub id: Uuid,
    pub name: String,
    pub host_id: Uuid,
    pub players: Vec<Player>,
    pub connections: HashMap<Uuid, Tx>,
    pub max_players: usize,
    pub game_state: GameState,
}

impl GameRoom {
    pub fn new(id: Uuid, name: String, host_id: Uuid) -> Self {
        Self {
            id,
            name,
            host_id,
            players: Vec::new(),
            connections: HashMap::new(),
            max_players: 10,
            game_state: GameState::Waiting,
        }
    }

    pub fn add_player(&mut self, player: Player, tx: Tx) -> Result<(), String> {
        if self.players.len() >= self.max_players {
            return Err("Room is full".to_string());
        }
        if self.game_state != GameState::Waiting {
            return Err("Game already started".to_string());
        }
        self.connections.insert(player.user_id, tx);
        self.players.push(player);
        Ok(())
    }

    pub fn remove_player(&mut self, user_id: &Uuid) {
        self.players.retain(|p| p.user_id != *user_id);
        self.connections.remove(user_id);
    }

    pub fn get_player_mut(&mut self, user_id: &Uuid) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.user_id == *user_id)
    }

    pub async fn broadcast(&self, message: Message, exclude: Option<Uuid>) {
        for (user_id, tx) in &self.connections {
            if let Some(excluded_id) = exclude {
                if user_id == &excluded_id {
                    continue;
                }
            }
            let _ = tx.send(message.clone());
        }
    }

    pub fn start_game(&mut self) -> Result<(), String> {
        if self.players.len() < 3 {
            return Err("Not enough players (minimum 3)".to_string());
        }
        self.game_state = GameState::Starting;
        Ok(())
    }
}
