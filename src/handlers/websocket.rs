use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use std::sync::Arc;
use futures::{SinkExt, StreamExt};
use redis::AsyncCommands;
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::{
    middleware::Player as PlayerAuth,
    state::AppState,
    websocket::{ClientMessage, ServerMessage, RoomState},
    models::Player,
};
use crate::dto::{PlayerDetails, PlayerWithRole, RoleResponse};
use crate::handlers::assign_roles;
use crate::models::Role;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    player: PlayerAuth,
) -> Response {
    let user_id = player.0.id();
    let username = player.0.username().to_string();
    ws.on_upgrade(move |socket| handle_socket(socket, state, user_id, username))
}

async fn handle_socket(socket: WebSocket, state: AppState, user_id: Uuid, username: String) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let room_state = Arc::new(state.room_state.clone());
    let room_state_clone = room_state.clone();
    let tx_clone = tx.clone();
    let redis_clone = state.redis.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                let text_str = text.to_string();
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text_str) {
                    handle_client_message(
                        client_msg,
                        user_id,
                        &username,
                        &room_state_clone,
                        &tx_clone,
                        redis_clone.clone(),
                    )
                    .await;
                }
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    cleanup_user_connections(user_id, &room_state).await;
}

async fn handle_client_message(
    msg: ClientMessage,
    user_id: Uuid,
    username: &str,
    room_state: &Arc<RoomState>,
    tx: &mpsc::UnboundedSender<Message>,
    mut redis: redis::aio::ConnectionManager,
) {
    match msg {
        ClientMessage::CreateRoom { room_name } => {
            let mut rooms = room_state.rooms.write().await;
            let room_id = Uuid::new_v4();
            let room = crate::websocket::GameRoom::new(room_id, room_name.clone(), user_id);
            rooms.insert(room_id, room);

            let msg = ServerMessage::RoomCreated { room_id, room_name };
            let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
        }
        ClientMessage::JoinRoom { room_id } => {
            let mut rooms = room_state.rooms.write().await;
            if let Some(room) = rooms.get_mut(&room_id) {
                let player = Player::new(user_id, username.to_string());
                let is_reconnect =
                    room.players.iter().any(|p| p.user_id == user_id) || room.host_id == user_id;

                if let Err(e) = room.add_player(player.clone(), tx.clone()) {
                    let error_msg = ServerMessage::Error { message: e };
                    let _ = tx.send(Message::Text(
                        serde_json::to_string(&error_msg).unwrap().into(),
                    ));
                    return;
                }
                let mut players: Vec<PlayerDetails> = Vec::new();
                for p in room.players.iter() {
                    players.push(PlayerDetails {
                        id: p.id,
                        user_id: p.user_id,
                        username: p.username.clone(),
                        role_id: p.role_id,
                        is_ready: p.is_ready,
                    });
                }
                let join_msg = ServerMessage::RoomJoined {
                    room_id,
                    room_name: room.name.clone(),
                    host_id: room.host_id,
                    max_players: room.max_players,
                    players,
                    game_state: room.game_state.clone(),
                };
                let _ = tx.send(Message::Text(
                    serde_json::to_string(&join_msg).unwrap().into(),
                ));

                if !is_reconnect {
                    let player_details: PlayerDetails = PlayerDetails {
                        id: player.id,
                        username: player.username,
                        is_ready: player.is_ready,
                        user_id: player.user_id,
                        role_id: player.role_id,
                    };
                    let broadcast_msg = ServerMessage::PlayerJoined {
                        room_id,
                        player: player_details,
                    };

                    room.broadcast(
                        Message::Text(serde_json::to_string(&broadcast_msg).unwrap().into()),
                        Some(user_id),
                    )
                    .await;
                }
            } else {
                let error_msg = ServerMessage::Error {
                    message: "Room not found".to_string(),
                };
                let _ = tx.send(Message::Text(
                    serde_json::to_string(&error_msg).unwrap().into(),
                ));
            }
        }
        ClientMessage::LeaveRoom { room_id } => {
            let mut rooms = room_state.rooms.write().await;
            if let Some(room) = rooms.get_mut(&room_id) {
                room.remove_player(&user_id);

                let left_msg = ServerMessage::RoomLeft { room_id };
                let _ = tx.send(Message::Text(
                    serde_json::to_string(&left_msg).unwrap().into(),
                ));

                let broadcast_msg = ServerMessage::PlayerLeft { room_id, user_id };
                room.broadcast(
                    Message::Text(serde_json::to_string(&broadcast_msg).unwrap().into()),
                    None,
                )
                .await;

                if room.players.is_empty() {
                    rooms.remove(&room_id);
                }
            }
        }
        ClientMessage::GetRoomState { room_id } => {
            let rooms = room_state.rooms.read().await;
            if let Some(room) = rooms.get(&room_id) {
                let mut players: Vec<PlayerDetails> = Vec::new();
                for p in room.players.iter() {
                    players.push(PlayerDetails {
                        id: p.id,
                        user_id: p.user_id,
                        username: p.username.clone(),
                        role_id: p.role_id,
                        is_ready: p.is_ready,
                    });
                }
                let state_msg = ServerMessage::RoomJoined {
                    room_id,
                    players,
                    room_name: room.name.clone(),
                    host_id: room.host_id,
                    max_players: room.max_players,
                    game_state: room.game_state.clone(),
                };
                let _ = tx.send(Message::Text(
                    serde_json::to_string(&state_msg).unwrap().into(),
                ));
            } else {
                let error_msg = ServerMessage::Error {
                    message: "Room not found".to_string(),
                };
                let _ = tx.send(Message::Text(
                    serde_json::to_string(&error_msg).unwrap().into(),
                ));
            }
        }
        ClientMessage::ToggleReady { room_id } => {
            let mut rooms = room_state.rooms.write().await;
            if let Some(room) = rooms.get_mut(&room_id) {
                if room.host_id == user_id {
                    return;
                }
                if let Some(player) = room.get_player_mut(&user_id) {
                    player.toggle_ready();
                    let is_ready = player.is_ready;

                    let msg = ServerMessage::PlayerReady {
                        room_id,
                        user_id,
                        is_ready,
                    };
                    room.broadcast(
                        Message::Text(serde_json::to_string(&msg).unwrap().into()),
                        None,
                    )
                    .await;
                }
            }
        }
        ClientMessage::SendMessage { room_id, message } => {
            let rooms = room_state.rooms.read().await;
            if let Some(room) = rooms.get(&room_id) {
                let msg = ServerMessage::Message {
                    room_id,
                    user_id,
                    username: username.to_string(),
                    message,
                };
                room.broadcast(
                    Message::Text(serde_json::to_string(&msg).unwrap().into()),
                    None,
                )
                .await;
            }
        }
        ClientMessage::StartGame { room_id } => {
            let mut rooms = room_state.rooms.write().await;
            if let Some(room) = rooms.get_mut(&room_id) {
                if room.host_id != user_id {
                    let error_msg = ServerMessage::Error {
                        message: "Only host can start game".to_string(),
                    };
                    let _ = tx.send(Message::Text(
                        serde_json::to_string(&error_msg).unwrap().into(),
                    ));
                    return;
                }

                if let Err(e) = room.start_game() {
                    let error_msg = ServerMessage::Error { message: e };
                    let _ = tx.send(Message::Text(
                        serde_json::to_string(&error_msg).unwrap().into(),
                    ));
                    return;
                }

                let msg = ServerMessage::GameStarting { room_id };
                room.broadcast(
                    Message::Text(serde_json::to_string(&msg).unwrap().into()),
                    None,
                )
                .await;

                let player_user_ids: Vec<Uuid> = room.players.iter().map(|p| p.user_id).collect();

                match assign_roles(&mut redis, player_user_ids).await {
                    Ok(assignments) => {
                        for assignment in assignments {
                            if let Some(player) = room
                                .players
                                .iter_mut()
                                .find(|p| p.user_id == assignment.player_id)
                            {
                                player.assign_role(assignment.role_id);

                                if let Some(player_tx) = room.connections.get(&assignment.player_id)
                                {
                                    let role_msg = ServerMessage::RoleAssigned {
                                        role_id: assignment.role_id,
                                    };
                                    let _ = player_tx.send(Message::Text(
                                        serde_json::to_string(&role_msg).unwrap().into(),
                                    ));
                                }
                            }
                        }

                        if let Some(host_tx) = room.connections.get(&room.host_id) {
                            let mut players_with_roles = Vec::new();

                            for p in room.players.iter() {
                                let role = if let Some(role_id) = p.role_id {
                                    let key = format!("role:{}", role_id);
                                    let role_data: Option<String> =
                                        redis.get(&key).await.ok().flatten();

                                    if let Some(data) = role_data {
                                        serde_json::from_str::<Role>(&data).ok().map(
                                            |r| RoleResponse {
                                                id: r.id,
                                                name: r.name,
                                                slug: r.slug,
                                                description: r.description,
                                                image: r.image,
                                                role_type: r.role_type,
                                                priority: r.priority,
                                            },
                                        )
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                players_with_roles.push(PlayerWithRole {
                                    id: p.id,
                                    user_id: p.user_id,
                                    username: p.username.clone(),
                                    role_id: p.role_id,
                                    is_ready: p.is_ready,
                                    role,
                                });
                            }

                            let host_msg = ServerMessage::AllRolesAssigned {
                                players: players_with_roles,
                            };
                            let _ = host_tx.send(Message::Text(
                                serde_json::to_string(&host_msg).unwrap().into(),
                            ));
                        }
                    }
                    Err(e) => {
                        let error_msg = ServerMessage::Error {
                            message: format!("Failed to assign roles: {}", e),
                        };
                        room.broadcast(
                            Message::Text(serde_json::to_string(&error_msg).unwrap().into()),
                            None,
                        )
                        .await;
                    }
                }
            }
        }
        ClientMessage::RemovePlayer {
            room_id,
            user_id: target_user_id,
        } => {
            let mut rooms = room_state.rooms.write().await;
            if let Some(room) = rooms.get_mut(&room_id) {
                if room.host_id != user_id {
                    let error_msg = ServerMessage::Error {
                        message: "Only host can remove players".to_string(),
                    };
                    let _ = tx.send(Message::Text(
                        serde_json::to_string(&error_msg).unwrap().into(),
                    ));
                    return;
                }

                if target_user_id == user_id {
                    let error_msg = ServerMessage::Error {
                        message: "Cannot remove yourself".to_string(),
                    };
                    let _ = tx.send(Message::Text(
                        serde_json::to_string(&error_msg).unwrap().into(),
                    ));
                    return;
                }

                let kicked_msg = ServerMessage::PlayerKicked {
                    room_id,
                    user_id: target_user_id,
                };
                room.broadcast(
                    Message::Text(serde_json::to_string(&kicked_msg).unwrap().into()),
                    None,
                )
                .await;

                room.remove_player(&target_user_id);
            } else {
                let error_msg = ServerMessage::Error {
                    message: "Room not found".to_string(),
                };
                let _ = tx.send(Message::Text(
                    serde_json::to_string(&error_msg).unwrap().into(),
                ));
            }
        }
        ClientMessage::ReassignRoles {
          room_id
        } => {
            let mut rooms = room_state.rooms.write().await;
            if let Some(room) = rooms.get_mut(&room_id) {
                if room.host_id != user_id {
                      let error_msg = ServerMessage::Error {
                          message: "Only host can reassign roles".to_string(),
                      };
                      let _ = tx.send(Message::Text(
                          serde_json::to_string(&error_msg).unwrap().into(),
                      ));
                      return;
                }

                room.reset_for_new_game();
                
                let game_starting_msg = ServerMessage::GameStarting { room_id };
                room.broadcast(
                    Message::Text(serde_json::to_string(&game_starting_msg).unwrap().into()),
                    None,
                )
                .await;
                
                let player_user_ids: Vec<Uuid > = room.players.iter().map(|p| p.user_id).collect();

                match assign_roles(&mut redis, player_user_ids).await  {
                  Ok(assignments) => {
                    for assignment in assignments {
                      if let Some(player) = room
                        .players
                        .iter_mut()
                        .find(|p| p.user_id == assignment.player_id)
                      {
                        player.assign_role(assignment.role_id);

                        if let Some(player_tx) = room.connections.get(&assignment.player_id) {
                          let role_msg = ServerMessage::RoleAssigned {
                            role_id: assignment.role_id,
                          };
                          let _ = player_tx.send(Message::Text(
                            serde_json::to_string(&role_msg).unwrap().into(),
                          ));
                        }
                      }
                    }

                    if let Some(host_tx) = room.connections.get(&room.host_id) {
                      let mut players_with_roles = Vec::new();

                      for p in room.players.iter() {
                        let role = if let Some(role_id) = p.role_id {
                          let key = format!("role:{}", role_id);
                          let role_data: Option<String> =
                            redis.get(&key).await.ok().flatten();

                          if let Some(data) = role_data {
                            serde_json::from_str::<Role>(&data).ok().map(
                              |r| RoleResponse {
                                id: r.id,
                                name: r.name,
                                slug: r.slug,
                                description: r.description,
                                image: r.image,
                                role_type: r.role_type,
                                priority: r.priority,
                              },
                            )
                          } else {
                            None
                          }
                        } else {
                          None
                        };

                        players_with_roles.push(PlayerWithRole {
                          id: p.id,
                          user_id: p.user_id,
                          username: p.username.clone(),
                          role_id: p.role_id,
                          is_ready: p.is_ready,
                          role,
                        });
                      }
                      let host_msg = ServerMessage::AllRolesAssigned {
                        players: players_with_roles,
                      };

                      let _ = host_tx.send(Message::Text(
                        serde_json::to_string(&host_msg).unwrap().into(),
                      ));
                    }
                  }
                  Err(e ) => {
                    let error_msg = ServerMessage::Error {
                      message: format!("Failed to assign roles: {}", e),
                    };
                    room.broadcast(
                      Message::Text(serde_json::to_string(&error_msg).unwrap().into()),
                      None,
                    )
                    .await;
                  }
                }
            }
        }
    }
}

async fn cleanup_user_connections(user_id: Uuid, room_state: &Arc<RoomState>) {
    let mut rooms = room_state.rooms.write().await;
    let room_ids: Vec<Uuid> = rooms.keys().copied().collect();

    for room_id in room_ids {
        if let Some(room) = rooms.get_mut(&room_id) {
            room.remove_player(&user_id);

            let msg = ServerMessage::PlayerLeft { room_id, user_id };
            room.broadcast(
                Message::Text(serde_json::to_string(&msg).unwrap().into()),
                None,
            )
            .await;

            if room.players.is_empty() {
                rooms.remove(&room_id);
            }
        }
    }
}
