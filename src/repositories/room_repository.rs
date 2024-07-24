use std::{error::Error, sync::Arc};

use tokio::sync::RwLock;

use crate::{models::{Room, Team}, my_state::MyState, types::GameStage};

pub async fn get_room_by_id(state: Arc<RwLock<MyState>>, room_id: i32) -> Result<Room, Box<dyn Error>> {
    let pool = &state.read().await.pool;
    let room = sqlx::query_as!(Room, "SELECT * FROM rooms WHERE id = $1", room_id)
        .fetch_one(pool)
        .await?;
    Ok(room)
}

pub async fn advance_room_game_stage(state: Arc<RwLock<MyState>>, room_id: i32) -> Result<(), Box<dyn Error>> {
    let pool = &state.read().await.pool;
    let Room {  game_stage ,..} = get_room_by_id(state.clone(), room_id).await?;
    sqlx::query!(
        "UPDATE rooms SET game_stage = $1 WHERE id = $2",
        game_stage.next().to_string(),
        room_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn change_room_current_team(state: Arc<RwLock<MyState>>, room_id: i32) -> Result<(), Box<dyn Error>> {
    let pool = &state.read().await.pool;
    let Room {  current_team ,..} = get_room_by_id(state.clone(), room_id).await?;
    let next_team = match current_team {
        Team::Red => Team::Blue,
        Team::Blue => Team::Red,
        _ => Team::Red,
    };
    sqlx::query!(
        "UPDATE rooms SET current_team = $1 WHERE id = $2",
        next_team.to_string(),
        room_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
