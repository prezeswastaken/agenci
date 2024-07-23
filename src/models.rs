use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Room {
    pub id: i32,
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
