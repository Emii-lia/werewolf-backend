use redis::aio::ConnectionManager;
use crate::configs::Config;
use crate::websocket::RoomState;

#[derive(Clone)]
pub struct AppState {
    pub redis: ConnectionManager,
    pub config: Config,
    pub room_state: RoomState,
}

impl AppState {
    pub fn new(redis: ConnectionManager, config: Config) -> Self {
        Self {
            redis,
            config,
            room_state: RoomState::new(),
        }
    }
}
