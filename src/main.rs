use agenci::{
    models::{Field, Room, Team},
    words::WORDS,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use rand::thread_rng;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use serde_json::{json, Value};
use socketioxide::{
    extract::{Bin, Data, SocketRef},
    SocketIo,
};
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;

async fn hello_world() -> &'static str {
    "Hello, chuju!"
}

#[axum_macros::debug_handler]
async fn add_room_handler(state: State<MyState>) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as!(
        Room,
        "INSERT INTO rooms DEFAULT VALUES RETURNING id, created_at"
    )
    .fetch_one(&state.pool)
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

            query.push_str(" ON CONFLICT DO NOTHING"); // Optional: handle duplicates

            let mut query_builder = sqlx::query(&query);

            for (room_id, text, team) in params {
                query_builder = query_builder.bind(room_id).bind(text).bind(team);
            }

            query_builder
                .execute(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            Ok((StatusCode::CREATED, Json(room)))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn get_rooms_handler(state: State<MyState>) -> Result<impl IntoResponse, impl IntoResponse> {
    // let query = "SELECT * FROM rooms";
    // match sqlx::query_as::<_, Room>(query)
    //     .fetch_all(&state.pool)
    //     .await
    // {
    //     Ok(rooms) => Ok(Json(rooms)),
    //     Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    // }
    match sqlx::query_as!(Room, "SELECT * FROM rooms")
        .fetch_all(&state.pool)
        .await
    {
        Ok(rooms) => Ok(Json(rooms)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

fn on_connect(socket: SocketRef, Data(data): Data<Value>) {
    info!("Socket.IO connected: {:?} {:?}", socket.ns(), socket.id);
    socket.emit("auth", data).ok();

    socket.on(
        "message",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            info!("Received event: {:?} {:?}", data, bin);
            socket.emit("message-back", data).ok();
        },
    );

    socket.on("join-room", |socket: SocketRef, Data::<Value>(data)| {
        info!("Joining room: {:?}", data);
        socket.join("1").ok();
        let rooms = socket.rooms().unwrap();
        let room = rooms.get(0).unwrap();
        socket
            .to(room.to_string())
            .broadcast()
            .emit("hello", json!({ "hello": "world" }))
            .ok();
    });
}

#[derive(Clone)]
struct MyState {
    pub pool: PgPool,
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    let state = MyState { pool };

    let results = sqlx::query_as!(Field, "SELECT * FROM fields")
        .fetch_all(&state.pool)
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
    io.ns("/", on_connect);

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/room", post(add_room_handler))
        .route("/rooms", get(get_rooms_handler))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(layer),
        )
        .with_state(state);

    Ok(router.into())
}
