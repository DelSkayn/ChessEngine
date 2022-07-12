use std::{
    borrow::BorrowMut,
    cell::RefCell,
    io,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use axum::{extract::Multipart, Extension, Json};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncWriteExt};
use tracing::info;

use crate::{error::Error, session::AdminSession, ApiResult, Context};

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

pub struct TempFile(Option<fs::File>, PathBuf);

impl TempFile {
    pub async fn create<S: AsRef<Path>>(p: S) -> Result<Self, io::Error> {
        let path = p.as_ref().to_path_buf();
        let f: fs::File = fs::File::create(&path).await?;
        Ok(TempFile(Some(f), path))
    }

    pub fn unwrap(mut self) -> fs::File {
        self.0.take().unwrap()
    }
}

impl Deref for TempFile {
    type Target = fs::File;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl DerefMut for TempFile {
    fn deref_mut(&mut self) -> &mut fs::File {
        self.0.as_mut().unwrap()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if self.0.is_some() {
            let p = self.1.clone();
            tokio::spawn(async move {
                fs::remove_file(p).await.ok();
            });
        }
    }
}

pub async fn create(
    mut multipart: Multipart,
    Extension(ext): Extension<Context>,
    AdminSession(user): AdminSession,
) -> ApiResult<Json<CreateEngineRes>> {
    let mut name: Option<String> = None;
    let mut previous_version: Option<i32> = None;
    let mut description: Option<String> = None;

    let mut file: Option<(TempFile, String)> = None;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| Error::BadRequest)?
    {
        let field_name = field.name().unwrap_or("");
        match field_name {
            "name" => {
                let tmp_name = field.text().await.map_err(|_| Error::BadRequest)?;
                if tmp_name
                    .chars()
                    .any(|x| !x.is_alphanumeric() && x != ' ' && x != '_' && x != '-' && x != '.')
                {
                    return Err(Error::BadRequest.into());
                }
                name = Some(tmp_name);
            }
            "description" => {
                description = Some(field.text().await.map_err(|_| Error::BadRequest)?);
            }
            "file" => {
                tokio::fs::create_dir_all("/tmp/chess_server/").await?;
                let mut bytes = [0u8; 32];
                rand::thread_rng().fill(&mut bytes);
                let name = base64::encode(&bytes);
                let file_path = format!("/tmp/chess_server/{}", &name);
                let mut tmp_file = TempFile::create(file_path).await?;
                while let Some(chunk) = field.chunk().await.map_err(|_| Error::BadRequest)? {
                    tmp_file.write_all(&chunk).await?;
                }
                file = Some((tmp_file, name))
            }
            _ => {}
        }
    }

    if name.is_none() || file.is_none() {
        return Err(Error::BadRequest.into());
    }

    tokio::fs::create_dir_all("./engines").await?;

    Ok(Json(CreateEngineRes::Ok { id: 0 }))
}
