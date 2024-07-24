use std::{error::Error, sync::Arc};

use tokio::sync::RwLock;

use crate::{models::Field, my_state::MyState};

pub async fn get_all_fields(state: Arc<RwLock<MyState>>) -> Result<Vec<Field>, Box<dyn Error>> {
    let pool = &state.read().await.pool;
    let fields = sqlx::query_as!(Field, "SELECT * FROM fields")
        .fetch_all(pool)
        .await?;
    Ok(fields)
}

pub async fn get_fields_for_room_id(
    state: Arc<RwLock<MyState>>,
    room_id: i32,
) -> Result<Vec<Field>, Box<dyn Error>> {
    let pool = &state.read().await.pool;
    let fields = sqlx::query_as!(Field, "SELECT * FROM fields WHERE room_id = $1", room_id)
        .fetch_all(pool)
        .await?;
    Ok(fields)
}

pub async fn get_field_by_id(
    state: Arc<RwLock<MyState>>,
    field_id: i32,
) -> Result<Option<Field>, Box<dyn Error>> {
    let pool = &state.read().await.pool;
    let field = sqlx::query_as!(Field, "SELECT * FROM fields WHERE id = $1", field_id)
        .fetch_optional(pool)
        .await?;
    Ok(field)
}

pub async fn mark_field_as_used(state: Arc<RwLock<MyState>>, field_id: i32) -> Result<(), Box<dyn Error>> {
    let pool = &state.read().await.pool;
    sqlx::query!(
        "UPDATE fields SET is_used = true WHERE id = $1",
        field_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
