use std::{fs::Permissions, num::TryFromIntError, os::unix::prelude::PermissionsExt, path::Path};

use axum::{
    extract::{Multipart, State},
    Form, Json,
};
use chess_core::uci::Version;
use common::engine::{self, Engine, UploadResponse};
use rand::Rng;
use tokio::{fs, io::AsyncWriteExt};
use tracing::{debug, error, info, warn};

use crate::{
    engine::retrieve_engine_info,
    error::{Error, ErrorContext, ErrorKind, ResultExt},
    session::AdminSession,
    temp::TempFile,
    Pool, BASE64_ENGINE,
};

#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "version")]
pub struct SqlxVersion {
    major: i32,
    minor: i32,
    patch: i32,
}

impl TryFrom<Version> for SqlxVersion {
    type Error = TryFromIntError;

    fn try_from(value: Version) -> Result<Self, Self::Error> {
        Ok(SqlxVersion {
            major: value.major.try_into()?,
            minor: value.minor.try_into()?,
            patch: value.patch.try_into()?,
        })
    }
}

pub async fn create(
    State(db): State<Pool>,
    AdminSession(user): AdminSession,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, Error> {
    let mut description: Option<String> = None;

    let mut file: Option<TempFile> = None;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| ErrorKind::BadRequest)?
    {
        let field_name = field.name().unwrap_or("");
        match field_name {
            "description" => {
                description = Some(field.text().await.map_err(|_| ErrorKind::BadRequest)?);
            }
            "file" => {
                let dir_path = Path::new("/tmp").join("chess_server");
                tokio::fs::create_dir_all(&dir_path)
                    .await
                    .context("creating temp upload directory")?;
                let mut bytes = [0u8; 32];
                rand::thread_rng().fill(&mut bytes);
                let name = base64::encode_engine(&bytes, &BASE64_ENGINE);
                let file_path = dir_path.join(&name);
                let mut tmp_file = TempFile::create(&file_path)
                    .await
                    .with_context(|| format!("creating temp file {}", file_path.display()))?;
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .log_error_debug()
                    .map_err(|_| ErrorKind::BadRequest)
                    .context("invalid file")?
                {
                    tmp_file.write_all(&chunk).await?;
                }
                file = Some(tmp_file)
            }
            _ => {}
        }
    }

    let file = file.ok_or(ErrorKind::BadRequest).context("missing file")?;

    let db = db.clone();

    tokio::spawn(async move {
        let engine_file = file;

        let (f, path) = engine_file.unwrap();
        std::mem::drop(f);

        debug!("initializing engine at {}", path.display());

        if let Err(e) = tokio::fs::set_permissions(&path, Permissions::from_mode(0o777)).await {
            warn!("failed to set permissions for engine file: {e}");
            return;
        }
        let info = match retrieve_engine_info(&path).await {
            Ok(info) => info,
            Err(e) => {
                warn!("failed to retrieve information from engine: {e}");
                return;
            }
        };

        let file_name = path.file_stem().expect("path should have a file stem");
        let new_path = Path::new("./engines").join(file_name);
        let file_name = file_name
            .to_str()
            .expect("filename is generated with ascii characters so should be a valid string")
            .to_owned();

        if let Err(e) = tokio::fs::copy(&path, new_path).await {
            warn!("failed to move engine file: {e}");
            return;
        }

        tokio::fs::remove_file(path).await.ok();

        let version = match info.version.map(SqlxVersion::try_from).transpose() {
            Ok(x) => x,
            Err(e) => {
                warn!("engine version did not fit in database: {e}");
                return;
            }
        };

        sqlx::query_as::<_, (i32,)>(
            r#"insert into "engine"
                (name, author ,description,engine_file, version, options, uploaded_by) 
                values ($1,$2,$3,$4,$5,$6,$7) returning engine_id"#,
        )
        .bind(info.name)
        .bind(info.author)
        .bind(description)
        .bind(file_name)
        .bind(version)
        .bind(sqlx::types::Json(info.options))
        .bind(user)
        .fetch_one(&db)
        .await
        .log_error()
        .ok();
    });

    Ok(Json(UploadResponse::Ok))
}
pub async fn get(State(db): State<Pool>) -> Result<Json<Vec<Engine>>, Error> {
    let res = sqlx::query_as!(
        Engine,
        r#"select engine_id as id,name,author,description,elo,games_played from "engine" "#,
    )
    .fetch_all(&db)
    .await
    .log_error()?;

    Ok(Json(res))
}

pub async fn delete(
    State(db): State<Pool>,
    _: AdminSession,
    Form(engine): Form<engine::DeleteReq>,
) -> Result<Json<engine::DeleteRes>, Error> {
    let path: String = sqlx::query_scalar!(
        r#"delete from "engine" where engine_id=$1 returning engine_file"#,
        engine.id
    )
    .fetch_one(&db)
    .await?;

    tokio::spawn(async move {
        let path = Path::new("engines").join(path);
        info!("removing engine: {}", path.display());
        if let Err(e) = fs::remove_file(&path).await {
            error!(
                "failed to remove engine with path `{}`: {e}",
                path.display()
            );
        }
    });

    Ok(Json(engine::DeleteRes::Ok))
}
