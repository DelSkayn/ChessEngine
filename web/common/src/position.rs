use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateReq {
    pub name: String,
    pub fen: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum CreateRes {
    Ok {
        id: i32,
    },
    Err {
        error: String,
        context: Option<Vec<String>>,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: i32,
    pub name: Option<String>,
    pub fen: String,
}
