use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub enum LoginResponse {
    Ok {
        token: String,
    },
    Err {
        error: String,
        context: Option<Vec<String>>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub enum RegisterResponse {
    Ok,
    Err {
        error: String,
        context: Option<Vec<String>>,
    },
}

#[derive(Serialize)]
pub enum GetResponse {
    Ok {
        username: String,
        is_admin: bool,
    },
    Err {
        error: String,
        context: Option<Vec<String>>,
    },
}
