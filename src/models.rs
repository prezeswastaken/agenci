use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Room {
    pub id: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Team {
    #[serde(rename = "red")]
    Red,
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "neutral")]
    Neutral,
    #[serde(rename = "black")]
    Black,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Field {
    pub id: i32,
    pub room_id: i32,
    team: Team,
    text: String,
    is_used: bool,
    pub created_at: chrono::NaiveDateTime,
}
