use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema,PartialEq)]
pub enum GameState {
    Waiting,
    Starting,
    InProgress,
    Finished,
}