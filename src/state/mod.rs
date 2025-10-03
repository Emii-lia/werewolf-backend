use redis::aio::MultiplexedConnection;
use crate::configs::Config;

#[derive(Clone)]
pub struct AppState {
    pub redis: MultiplexedConnection,
    pub config: Config,
}

impl AppState {
    pub fn new(redis: MultiplexedConnection, config: Config) -> Self {
        Self { redis, config }
    }
}
