use agenci::{models::Room, words::WORDS};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use socketioxide::{
    extract::{Bin, Data, SocketRef},
    SocketIo,
};
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;
use rand::seq::SliceRandom;
use rand::thread_rng;

async fn hello_world() -> &'static str {
    "Hello, chuju!"
}

async fn add_room_handler(state: State<MyState>) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as!(Room,  "INSERT INTO rooms DEFAULT VALUES RETURNING id, created_at")
        .fetch_one(&state.pool)
        .await
    {
        Ok(room) => {
            let words = WORDS.clone().map(|s| s.to_string());
            assert!(words.len() >= 25);

            let mut rng = thread_rng();
            let words = words.choose_multiple(&mut rng, 25).collect::<Vec<_>>();
            for words in words.iter() {
                sqlx::query_as!(
                    Field,
                    "INSERT INTO fields (room_id, text) VALUES ($1, $2)",
                    room.id,
                    words
                )
                .execute(&state.pool)
                .await
                .ok();
            };

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
        socket.to(room.to_string()).broadcast().emit("hello", json!({ "hello": "world" })).ok();
    });
}

#[derive(Clone)]
struct MyState {
    pool: PgPool,
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    let state = MyState { pool };

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
