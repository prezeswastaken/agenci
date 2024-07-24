use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinRoomRequest {
    pub player_id: i32,
    pub room_id: i32,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameStage {
    #[serde(rename = "waiting_for_players")]
    WaitingForPlayers,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "finished")]
    Finished,
}

impl From<String> for GameStage {
    fn from(s: String) -> Self {
        match s.as_str() {
            "waiting_for_players" => GameStage::WaitingForPlayers,
            "in_progress" => GameStage::InProgress,
            "finished" => GameStage::Finished,
            _ => panic!("Invalid game stage"),
        }
    }
}

impl GameStage {
    pub fn to_string(&self) -> String {
        match self {
            GameStage::WaitingForPlayers => "waiting_for_players".to_string(),
            GameStage::InProgress => "in_progress".to_string(),
            GameStage::Finished => "finished".to_string(),
        }
    }

    pub fn next(&self) -> GameStage {
        match self {
            GameStage::WaitingForPlayers => GameStage::InProgress,
            GameStage::InProgress => GameStage::Finished,
            GameStage::Finished => GameStage::Finished,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Role {
    #[serde(rename = "shower")]
    Shower,
    #[serde(rename = "guesser")]
    Guesser,
}

impl Role {
    pub fn to_string(&self) -> String {
        match self {
            Role::Shower => "shower".to_string(),
            Role::Guesser => "guesser".to_string(),
        }
    }
}

impl From<String> for Role {
    fn from(s: String) -> Self {
        match s.as_str() {
            "shower" => Role::Shower,
            "guesser" => Role::Guesser,
            _ => panic!("Invalid role"),
        }
    }
}

