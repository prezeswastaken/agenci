use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::types::{GameStage, Role};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Room {
    pub id: i32,
    pub game_stage: GameStage,
    pub current_team: Team,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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

impl Team {
    pub fn to_string(&self) -> String {
        match self {
            Team::Red => "red".to_string(),
            Team::Blue => "blue".to_string(),
            Team::Neutral => "neutral".to_string(),
            Team::Black => "black".to_string(),
        }
    }
}

impl From<String> for Team {
    fn from(s: String) -> Self {
        match s.as_str() {
            "red" => Team::Red,
            "blue" => Team::Blue,
            "neutral" => Team::Neutral,
            "black" => Team::Black,
            _ => panic!("Invalid team"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Field {
    pub id: i32,
    pub room_id: i32,
    pub team: Team,
    pub text: String,
    pub is_used: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Player {
    pub id: i32,
    pub room_id: i32,
    pub username: String,
    pub team: Team,
    pub role: Role,
    pub created_at: chrono::NaiveDateTime,
}
