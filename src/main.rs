use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};
use models::Room;
use sqlx::PgPool;

pub mod models;

async fn hello_world() -> &'static str {
    "Hello, chuju!"
}

async fn add_room_handler(state: State<MyState>) -> Result<impl IntoResponse, impl IntoResponse> {
    let query = "INSERT INTO rooms DEFAULT VALUES RETURNING id, created_at";
    match sqlx::query_as::<_, Room>(query)
        .bind("test")
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

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/room", post(add_room_handler))
        .route("/rooms", get(get_rooms_handler))
        .with_state(state);

    Ok(router.into())
}
