use std::{error::Error, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    models::{Field, Player},
    my_state::MyState,
};

pub async fn get_players_by_room_id(
    state: Arc<RwLock<MyState>>,
    room_id: i32,
) -> Result<Vec<Player>, Box<dyn Error>> {
    let pool = &state.read().await.pool;
    let teams = sqlx::query_as!(Player, "SELECT * FROM players WHERE room_id = $1", room_id)
        .fetch_all(pool)
        .await?;
    Ok(teams)
}

pub async fn is_player_id_in_room(
    state: Arc<RwLock<MyState>>,
    player_id: i32,
    room_id: i32,
) -> Result<bool, Box<dyn Error>> {
    let pool = &state.read().await.pool;
    let player = sqlx::query!(
        "SELECT * FROM players WHERE room_id = $1 AND id = $2",
        room_id,
        player_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(player.is_some())
}

pub async fn get_player_by_id(
    state: Arc<RwLock<MyState>>,
    player_id: i32,
) -> Result<Option<Player>, Box<dyn Error>> {
    let pool = &state.read().await.pool;
    let player = sqlx::query_as!(Player, "SELECT * FROM players WHERE id = $1", player_id)
        .fetch_optional(pool)
        .await?;
    Ok(player)
}

pub async fn create_player_for_the_room_id(
    state: Arc<RwLock<MyState>>,
    username: String,
    room_id: i32,
) -> Result<Player, Box<dyn Error>> {
    let pool = &state.read().await.pool;
    let player = sqlx::query_as!(
        Player,
        "INSERT INTO players (room_id, username) VALUES ($1, $2) RETURNING *",
        room_id,
        username
    )
    .fetch_one(pool)
    .await?;
    Ok(player)
}
