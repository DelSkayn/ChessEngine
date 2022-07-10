use axum::{Extension, Form};
use serde::{Deserialize, Serialize};

use crate::{session::AdminSession, ApiResult, Context};

#[derive(Deserialize)]
pub struct CreateEngineReq {
    name: Option<String>,
    previous_version: Option<u64>,
    description: String,
}

#[derive(Serialize)]
pub enum CreateEngineRes {
    Ok { id: u64 },
}

pub async fn create(
    Form(req): Form<CreateEngineReq>,
    Extension(ext): Extension<Context>,
    _: AdminSession,
) -> ApiResult<CreateEngineRes> {
    Ok(CreateEngineRes::Ok { id: 0 })
}
