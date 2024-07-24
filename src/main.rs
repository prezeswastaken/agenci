use std::sync::Arc;

use agenci::{
    models::{Field, Player, Room, Team},
    my_state::MyState,
    repositories::{
        field_repository::{get_all_fields, get_field_by_id, get_fields_for_room_id, mark_field_as_used},
        player_repository::{
            create_player_for_the_room_id, get_player_by_id, get_players_by_room_id,
            is_player_id_in_room,
        }, room_repository::get_room_by_id,
    },
    types::{JoinRoomRequest, Role},
    words::WORDS,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_macros::debug_handler;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use serde_json::{json, Value};
use socketioxide::{
    extract::{Bin, Data, SocketRef},
    SocketIo,
};
use sqlx::PgPool;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;

async fn hello_world() -> &'static str {
    "Hello, chuju!"
}

async fn check_field_handler(state: State<Arc<RwLock<MyState>>>, Path((field_id, player_id)): Path<(i32, i32)>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let player = get_player_by_id(state.clone().0, player_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?.map_or_else(|| Err((StatusCode::NOT_FOUND, "Player not found".to_string())), |p| Ok(p))?;
    let field = get_field_by_id(state.clone().0, field_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?.map_or_else(|| Err((StatusCode::NOT_FOUND, "Field not found".to_string())), |f| Ok(f))?;
    if player.role != Role::Guesser {
        return Err((StatusCode::FORBIDDEN, "Only guessers can check fields".to_string()));
    }
    mark_field_as_used(state.0, field_id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn get_room_by_room_id_handler(
    state: State<Arc<RwLock<MyState>>>,
    Path(room_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let room = get_room_by_id(state.0, room_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::OK, Json(room)))
}

async fn get_all_fields_handler(
    state: State<Arc<RwLock<MyState>>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let fields = get_all_fields(state.0)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::OK, Json(fields)))
}

async fn get_fields_for_room_id_handler(
    state: State<Arc<RwLock<MyState>>>,
    Path(room_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let fields = get_fields_for_room_id(state.0, room_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::OK, Json(fields)))
}

async fn is_player_in_room_handler(
    state: State<Arc<RwLock<MyState>>>,
    Path((room_id, player_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let is_player_in_room = is_player_id_in_room(state.0, player_id, room_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::OK, Json(is_player_in_room)))
}

async fn create_player_for_the_room_id_handler(
    state: State<Arc<RwLock<MyState>>>,
    Path((username, room_id)): Path<(String, i32)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let player = create_player_for_the_room_id(state.0, username, room_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(player)))
}

async fn get_player_by_id_handler(
    state: State<Arc<RwLock<MyState>>>,
    Path(player_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let player = get_player_by_id(state.0, player_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::OK, Json(player)))
}

#[debug_handler]
async fn get_players_for_room_handler(
    state: State<Arc<RwLock<MyState>>>,
    Path(room_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let players = get_players_by_room_id(state.0, room_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::OK, Json(players)))
}

#[axum_macros::debug_handler]
async fn add_room_handler(
    state: State<Arc<RwLock<MyState>>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let pool = &state.read().await.pool;
    match sqlx::query_as!(Room, "INSERT INTO rooms DEFAULT VALUES RETURNING *")
        .fetch_one(pool)
        .await
    {
        Ok(room) => {
            let words = WORDS.clone().map(|s| s.to_string());
            assert!(words.len() >= 25);

            let mut rng = StdRng::from_entropy();
            let words = words.choose_multiple(&mut rng, 25).collect::<Vec<_>>();
            let mut query = String::from("INSERT INTO fields (room_id, text, team) VALUES ");
            let mut params: Vec<(i32, String, String)> = Vec::new();

            // Create a vector of indices for all fields
            let mut indices: Vec<usize> = (0..words.len()).collect();
            let mut rng = StdRng::from_entropy();

            // Shuffle indices to randomize selection
            indices.shuffle(&mut rng);

            // Get indices for each team
            let red_indices = indices.iter().take(7).copied().collect::<Vec<_>>();
            let blue_indices = indices.iter().skip(7).take(6).copied().collect::<Vec<_>>();
            let black_index = indices.get(13).copied();
            let neutral_indices = indices.iter().skip(14).copied().collect::<Vec<_>>();

            for (i, word) in words.iter().enumerate() {
                if i > 0 {
                    query.push_str(", ");
                }
                query.push_str(&format!("(${}, ${}, ${})", i * 3 + 1, i * 3 + 2, i * 3 + 3));

                let team = if red_indices.contains(&i) {
                    "red"
                } else if blue_indices.contains(&i) {
                    "blue"
                } else if black_index == Some(i) {
                    "black"
                } else {
                    "neutral"
                };

                params.push((room.id, word.to_owned().clone(), team.to_string()));
            }

            query.push_str(" ON CONFLICT DO NOTHING");

            let mut query_builder = sqlx::query(&query);

            for (room_id, text, team) in params {
                query_builder = query_builder.bind(room_id).bind(text).bind(team);
            }

            query_builder
                .execute(pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            Ok((StatusCode::CREATED, Json(room)))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn get_rooms_handler(
    state: State<Arc<RwLock<MyState>>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // let query = "SELECT * FROM rooms";
    // match sqlx::query_as::<_, Room>(query)
    //     .fetch_all(&state.pool)
    //     .await
    // {
    //     Ok(rooms) => Ok(Json(rooms)),
    //     Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    // }
    let pool = &state.read().await.pool;
    match sqlx::query_as!(Room, "SELECT * FROM rooms")
        .fetch_all(pool)
        .await
    {
        Ok(rooms) => Ok(Json(rooms)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

fn on_connect(socket: SocketRef, Data(data): Data<Value>, state: Arc<RwLock<MyState>>) {
    info!("Socket.IO connected: {:?} {:?}", socket.ns(), socket.id);
    socket.emit("auth", data).ok();

    socket.on(
        "message",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            info!("Received event: {:?} {:?}", data, bin);
            socket.emit("message-back", data).ok();
        },
    );

    socket.on(
        "join-room",
        |socket: SocketRef, Data::<JoinRoomRequest>(join_room_request)| {
            let JoinRoomRequest {
                player_id,
                room_id,
                username,
            } = join_room_request;

            info!("Player with id {} is joining room: {}", player_id, room_id);

            socket.join(room_id.to_string()).ok();
            let rooms = socket.rooms().unwrap();
            let room = rooms.get(0).unwrap();
            socket
                .to(room.to_string())
                .broadcast()
                .emit(
                    "player-joined",
                    format!("{username} dołączył(a) do pokoju!"),
                )
                .ok();
        },
    );
    socket.on("field-updated", |socket: SocketRef| {
        let rooms = socket.rooms().unwrap();
        let room = rooms.get(0).unwrap();
        socket
            .to(room.to_string())
            .broadcast()
            .emit(
                "field-updated",
                None::<Value>
            )
            .ok();

    });
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    let state = MyState { pool };
    let state = Arc::new(RwLock::new(state));

    let results = sqlx::query_as!(Field, "SELECT * FROM fields")
        .fetch_all(&state.write().await.pool)
        .await
        .expect("Failed to fetch rooms");
    println!("{:?}", results);
    let text = results.iter().map(|f| &f.team).collect::<Vec<_>>();
    println!("{:?}", text);

    let red_count = results.iter().filter(|f| f.team == Team::Red).count();
    let blue_count = results.iter().filter(|f| f.team == Team::Blue).count();
    let black_count = results.iter().filter(|f| f.team == Team::Black).count();
    let neutral_count = results.iter().filter(|f| f.team == Team::Neutral).count();

    info!(
        "Red: {}, Blue: {}, Black: {}, Neutral: {}",
        red_count, blue_count, black_count, neutral_count
    );

    let (layer, io) = SocketIo::new_layer();
    let state_clone = state.clone();
    io.ns("/", move |socket, data| {
        on_connect(socket, data, state_clone.clone())
    });

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/room", post(add_room_handler))
        .route("/room", get(get_rooms_handler))
        .route("/fields", get(get_all_fields_handler))
        .route("/room/:room_id/players", get(get_players_for_room_handler))
        .route("/room/:room_id/fields", get(get_fields_for_room_id_handler))
        .route("/room/:room_id", get(get_room_by_room_id_handler))
        .route(
            "/is-player-in-room/:room_id/:player_id",
            get(is_player_in_room_handler),
        )
        .route(
            "/player/:username/room/:room_id",
            post(create_player_for_the_room_id_handler),
        )
        .route("/player/:player_id", get(get_player_by_id_handler))
        .route("/field/:field_id/player/:player_id", post(check_field_handler))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(layer),
        )
        .with_state(state);

    Ok(router.into())
}
