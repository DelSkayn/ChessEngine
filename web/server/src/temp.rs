use std::{
    env, io,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use tokio::fs;

use crate::BASE64_ENGINE;

pub struct TempDir {
    path: Option<PathBuf>,
}

impl TempDir {
    pub async fn new_prefixed(prefix: impl AsRef<Path>) -> Result<Self, io::Error> {
        let dir = env::temp_dir();
        let random: [u8; 16] = rand::random();
        let name = base64::encode_engine(random, &BASE64_ENGINE);
        let path = dir.join(prefix).join(name);

        tokio::fs::create_dir_all(&path).await?;

        Ok(TempDir { path: Some(path) })
    }

    pub fn keep(mut self) -> PathBuf {
        self.path.take().unwrap()
    }

    pub async fn delete(mut self) -> Result<(), io::Error> {
        tokio::fs::remove_dir_all(self.path.take().unwrap()).await
    }

    pub fn path(&self) -> &Path {
        self.path.as_ref().unwrap()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let path = self.path.take();

        if let Some(path) = path {
            tokio::spawn(async move {
                tokio::fs::remove_dir_all(path).await.ok();
            });
        }
    }
}

pub struct TempFile(Option<fs::File>, PathBuf);

impl TempFile {
    pub async fn create<S: AsRef<Path>>(p: S) -> Result<Self, io::Error> {
        let path = p.as_ref().to_path_buf();
        let f: fs::File = fs::File::create(&path).await?;
        Ok(TempFile(Some(f), path))
    }

    pub fn path(&self) -> &Path {
        &self.1
    }

    pub fn unwrap(mut self) -> (fs::File, PathBuf) {
        (self.0.take().unwrap(), self.1.clone())
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
