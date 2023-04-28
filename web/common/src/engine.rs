use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum UploadResponse {
    Ok,
    Err {
        error: String,
        context: Option<Vec<String>>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Engine {
    pub id: i32,
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub elo: f64,
    pub games_played: i32,
}

impl PartialEq for Engine {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.description == other.description
            && self.elo == other.elo
            && self.games_played == other.games_played
    }
}

impl Eq for Engine {}

#[derive(Serialize, Deserialize)]
pub struct DeleteReq {
    pub id: i32,
}

#[derive(Serialize, Deserialize)]
pub enum DeleteRes {
    Ok,
    Err {
        error: String,
        context: Option<Vec<String>>,
    },
}
