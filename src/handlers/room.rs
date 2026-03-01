use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use axum::extract::Path;
use uuid::Uuid;
use crate::{
    middleware::AuthUser,
    models::GameState,
    state::AppState,
    dto::{CreateRoomRequest, RoomDetails, RoomInfo},
};
use crate::dto::PlayerDetails;

#[utoipa::path(
    post,
    path = "/api/rooms",
    request_body = CreateRoomRequest,
    responses(
        (status = 201, description = "Room created successfully", body = RoomInfo),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "rooms"
)]
pub async fn create_room(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(data): Json<CreateRoomRequest>,
) -> Result<(StatusCode, Json<RoomInfo>), (StatusCode, String)> {
    let room_id = Uuid::new_v4();
    let mut rooms = state.room_state.rooms.write().await;

    let room = crate::websocket::GameRoom::new(room_id, data.name.clone(), data.max_players ,auth.0);

    let room_info = RoomInfo {
        id: room.id,
        name: room.name.clone(),
        host_id: room.host_id,
        player_count: room.players.len(),
        max_players: room.max_players,
        game_state: room.game_state.clone(),
    };

    rooms.insert(room_id, room);

    Ok((StatusCode::CREATED, Json(room_info)))
}

#[utoipa::path(
    get,
    path = "/api/rooms",
    responses(
        (status = 200, description = "List of available rooms", body = Vec<RoomInfo>)
    ),
    tag = "rooms"
)]
pub async fn get_rooms(
    State(state): State<AppState>,
) -> Result<Json<Vec<RoomInfo>>, (StatusCode, String)> {
    let rooms = state.room_state.rooms.read().await;

    let room_list: Vec<RoomInfo> = rooms
        .values()
        .filter(|room| room.game_state == GameState::Waiting)
        .map(|room| RoomInfo {
            id: room.id,
            name: room.name.clone(),
            host_id: room.host_id,
            player_count: room.players.len(),
            max_players: room.max_players,
            game_state: room.game_state.clone(),
        })
        .collect();

    Ok(Json(room_list))
}

#[utoipa::path(
    get,
    path = "/api/rooms/{room_id}",
    params(
        ("room_id" = Uuid, Path, description = "Room ID to fetch")
    ),
    responses(
        (status = 200, description = "Room details with players", body = RoomDetails),
        (status = 404, description = "Room not found")
    ),
    tag = "rooms"
)]
pub async fn get_room_details(
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<RoomDetails>, (StatusCode, String)> {
    let rooms = state.room_state.rooms.read().await;

    if let Some(room) = rooms.get(&room_id) {
        let mut players: Vec<PlayerDetails> = Vec::new();
        for player in room.players.iter() {
            players.push(PlayerDetails {
                id: player.id,
                user_id: player.user_id,
                username: player.username.clone(),
                role_id: player.role_id,
                is_ready: player.is_ready,
            });       
        }
        
        let details = RoomDetails {
            id: room.id,
            name: room.name.clone(),
            host_id: room.host_id,
            players,
            max_players: room.max_players,
            game_state: room.game_state.clone(),
        };
        Ok(Json(details))
    } else {
        Err((StatusCode::NOT_FOUND, "Room not found".to_string()))
    }
}
