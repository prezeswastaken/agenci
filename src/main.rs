use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};
use models::Room;
use serde_json::Value;
use socketioxide::{extract::{Bin, Data, SocketRef}, SocketIo};
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

pub mod models;

async fn hello_world() -> &'static str {
    "Hello, chuju!"
}

async fn add_room_handler(state: State<MyState>) -> Result<impl IntoResponse, impl IntoResponse> {
    let query = "INSERT INTO rooms DEFAULT VALUES RETURNING id, created_at";
    match sqlx::query_as::<_, Room>(query)
        .fetch_one(&state.pool)
        .await
    {
        Ok(room) => Ok((StatusCode::CREATED, Json(room))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn get_rooms_handler(state: State<MyState>) -> Result<impl IntoResponse, impl IntoResponse> {
    let query = "SELECT * FROM rooms";
    match sqlx::query_as::<_, Room>(query)
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
}

#[derive(Clone)]
struct MyState {
    pool: PgPool,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres]
    pool: PgPool,
) -> shuttle_axum::ShuttleAxum {

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
                .layer(layer)
        )
        .with_state(state);

    Ok(router.into())
}
