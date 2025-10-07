use axum::{
    routing::{get, post},
    Router,
    response::{IntoResponse, Html},
    http::StatusCode,
    middleware::from_fn_with_state,
};
use utoipa::OpenApi;
use crate::{
    handlers::{signup, login, create_guest_session},
    dto::{
        SignupRequest,
        LoginRequest,
        LoginResponse,
        UserResponse,
        RoleResponse,
        RoleCreateRequest,
        RoleUpdateRequest,
        CreateRoomRequest,
        RoomInfo,
        RoomDetails,
        PlayerDetails,
        CreateGuestRequest,
        GuestSessionResponse,
    },
    models::{GameState},
    state::AppState,
    middleware::{auth_middleware, inject_state_middleware},
};
use crate::handlers::{
    create_role, get_role_by_id, get_roles,
    get_user_by_id, get_user_by_username, get_users, verify_username_exists,
    update_role, ws_handler,
    create_room, get_rooms, get_room_details,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::signup,
        crate::handlers::login,
        crate::handlers::create_guest_session,
        crate::handlers::get_users,
        crate::handlers::get_user_by_id,
        crate::handlers::get_user_by_username,
        crate::handlers::verify_username_exists,
        crate::handlers::create_role,
        crate::handlers::get_roles,
        crate::handlers::get_role_by_id,
        crate::handlers::update_role,
        crate::handlers::create_room,
        crate::handlers::get_rooms,
        crate::handlers::get_room_details,
    ),
    components(schemas(
        UserResponse,
        SignupRequest,
        LoginRequest,
        LoginResponse,
        CreateGuestRequest,
        GuestSessionResponse,
        RoleCreateRequest,
        RoleResponse,
        RoleUpdateRequest,
        CreateRoomRequest,
        RoomInfo,
        RoomDetails,
        PlayerDetails,
        GameState,
    )),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "guest", description = "Guest session endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "roles", description = "Game roles management endpoints"),
        (name = "rooms", description = "Game room management endpoints")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    let user_routes = Router::new()
        .route("/api/users", get(get_users))
        .route("/api/users/search", get(get_user_by_username))
        .route("/api/users/{id}", get(get_user_by_id))
        .layer(from_fn_with_state(state.clone(), auth_middleware));

    let role_routes = Router::new()
        .route("/api/roles", get(get_roles).post(create_role))
        .route("/api/roles/{id}", get(get_role_by_id).put(update_role))
        .layer(from_fn_with_state(state.clone(), auth_middleware));

    let room_routes = Router::new()
        .route("/api/rooms", get(get_rooms).post(create_room))
        .route("/api/rooms/{room_id}", get(get_room_details))
        .layer(from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .route("/api-docs/openapi.json", get(serve_api_docs))
        .route("/swagger-ui", get(serve_swagger))
        .route("/health", get(health_check))
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        .route("/api/guest/session", post(create_guest_session))
        .route("/ws", get(ws_handler))
        .route("/api/users/verify/{username}", get(verify_username_exists))
        .merge(user_routes)
        .merge(role_routes)
        .merge(room_routes)
        .layer(from_fn_with_state(state.clone(), inject_state_middleware))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn serve_api_docs() -> impl IntoResponse {
    (StatusCode::OK, axum::Json(ApiDoc::openapi()))
}

async fn serve_swagger() -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <title>SwaggerUI</title>
            <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui.css" />
        </head>
        <body>
            <div id="swagger-ui"></div>
            <script src="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui-bundle.js" crossorigin></script>
            <script>
                window.onload = () => {{
                    window.ui = SwaggerUIBundle({{
                        url: '/api-docs/openapi.json',
                        dom_id: '#swagger-ui',
                    }});
                }};
            </script>
        </body>
        </html>"#
    ))
}
