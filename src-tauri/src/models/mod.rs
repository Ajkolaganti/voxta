use crate::error::{AppError, AppResult};
use dirs_next::data_dir;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::Sha256;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

const MODEL_BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";
const APP_DIR: &str = "Voxta";
const LEGACY_APP_DIR: &str = "LocalType";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChecksumAlgorithm {
    Sha1,
    Sha256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub disk_size: String,
    pub language_mode: String,
    pub checksum_algorithm: ChecksumAlgorithm,
    pub checksum: String,
    pub installed: bool,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadProgress {
    pub model_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub done: bool,
}

#[derive(Debug, Clone)]
struct ModelDefinition {
    id: &'static str,
    display_name: &'static str,
    file_name: &'static str,
    size_bytes: u64,
    disk_size: &'static str,
    language_mode: &'static str,
    checksum_algorithm: ChecksumAlgorithm,
    checksum: &'static str,
}

static MODELS: &[ModelDefinition] = &[
    ModelDefinition {
        id: "tiny",
        display_name: "Tiny multilingual",
        file_name: "ggml-tiny.bin",
        size_bytes: 75 * 1024 * 1024,
        disk_size: "75 MiB",
        language_mode: "multilingual",
        checksum_algorithm: ChecksumAlgorithm::Sha1,
        checksum: "bd577a113a864445d4c299885e0cb97d4ba92b5f",
    },
    ModelDefinition {
        id: "tiny.en",
        display_name: "Tiny English",
        file_name: "ggml-tiny.en.bin",
        size_bytes: 75 * 1024 * 1024,
        disk_size: "75 MiB",
        language_mode: "english",
        checksum_algorithm: ChecksumAlgorithm::Sha1,
        checksum: "c78c86eb1a8faa21b369bcd33207cc90d64ae9df",
    },
    ModelDefinition {
        id: "base",
        display_name: "Base multilingual",
        file_name: "ggml-base.bin",
        size_bytes: 142 * 1024 * 1024,
        disk_size: "142 MiB",
        language_mode: "multilingual",
        checksum_algorithm: ChecksumAlgorithm::Sha1,
        checksum: "465707469ff3a37a2b9b8d8f89f2f99de7299dac",
    },
    ModelDefinition {
        id: "base.en",
        display_name: "Base English",
        file_name: "ggml-base.en.bin",
        size_bytes: 142 * 1024 * 1024,
        disk_size: "142 MiB",
        language_mode: "english",
        checksum_algorithm: ChecksumAlgorithm::Sha1,
        checksum: "137c40403d78fd54d454da0f9bd998f78703390c",
    },
    ModelDefinition {
        id: "small",
        display_name: "Small multilingual",
        file_name: "ggml-small.bin",
        size_bytes: 466 * 1024 * 1024,
        disk_size: "466 MiB",
        language_mode: "multilingual",
        checksum_algorithm: ChecksumAlgorithm::Sha1,
        checksum: "55356645c2b361a969dfd0ef2c5a50d530afd8d5",
    },
    ModelDefinition {
        id: "small.en",
        display_name: "Small English",
        file_name: "ggml-small.en.bin",
        size_bytes: 466 * 1024 * 1024,
        disk_size: "466 MiB",
        language_mode: "english",
        checksum_algorithm: ChecksumAlgorithm::Sha1,
        checksum: "db8a495a91d927739e50b3fc1cc4c6b8f6c2d022",
    },
];

#[derive(Debug, Clone)]
pub struct ModelManager {
    models_dir: PathBuf,
    legacy_models_dir: Option<PathBuf>,
}

impl ModelManager {
    pub fn new() -> AppResult<Self> {
        let base = data_dir()
            .ok_or_else(|| AppError::Model("could not resolve app data directory".to_string()))?;
        let models_dir = base.join(APP_DIR).join("models");
        let legacy_models_dir = base.join(LEGACY_APP_DIR).join("models");
        fs::create_dir_all(&models_dir)?;
        Ok(Self {
            models_dir,
            legacy_models_dir: legacy_models_dir.exists().then_some(legacy_models_dir),
        })
    }

    #[cfg(test)]
    pub fn from_dir(models_dir: PathBuf) -> Self {
        Self {
            models_dir,
            legacy_models_dir: None,
        }
    }

    #[cfg(test)]
    pub fn from_dirs(models_dir: PathBuf, legacy_models_dir: PathBuf) -> Self {
        Self {
            models_dir,
            legacy_models_dir: Some(legacy_models_dir),
        }
    }

    pub fn models_dir(&self) -> &Path {
        &self.models_dir
    }

    pub fn list(&self) -> Vec<ModelInfo> {
        MODELS
            .iter()
            .map(|definition| self.to_info(definition))
            .collect()
    }

    pub fn model_path(&self, model_id: &str) -> AppResult<PathBuf> {
        let definition = find_model(model_id)?;
        Ok(self.models_dir.join(definition.file_name))
    }

    pub fn installed_model_path(&self, model_id: &str) -> AppResult<PathBuf> {
        let definition = find_model(model_id)?;
        if let Some(path) = self.installed_path_for_definition(definition) {
            Ok(path)
        } else {
            Err(AppError::Model(format!(
                "model '{model_id}' is not installed"
            )))
        }
    }

    pub async fn download<F>(&self, model_id: &str, mut on_progress: F) -> AppResult<ModelInfo>
    where
        F: FnMut(ModelDownloadProgress) + Send,
    {
        let definition = find_model(model_id)?;
        fs::create_dir_all(&self.models_dir)?;

        let url = format!("{MODEL_BASE_URL}/{}", definition.file_name);
        let target = self.models_dir.join(definition.file_name);
        let part = target.with_extension("bin.part");

        let result = async {
            let response = reqwest::get(url).await?.error_for_status()?;
            let total = response.content_length();
            let mut stream = response.bytes_stream();
            let mut file = fs::File::create(&part)?;
            let mut downloaded = 0_u64;

            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                file.write_all(&chunk)?;
                downloaded += chunk.len() as u64;
                on_progress(ModelDownloadProgress {
                    model_id: model_id.to_string(),
                    downloaded_bytes: downloaded,
                    total_bytes: total,
                    done: false,
                });
            }
            file.flush()?;

            verify_file(&part, definition.checksum_algorithm, definition.checksum)?;
            fs::rename(&part, &target)?;

            on_progress(ModelDownloadProgress {
                model_id: model_id.to_string(),
                downloaded_bytes: downloaded,
                total_bytes: total,
                done: true,
            });

            Ok(self.to_info(definition))
        }
        .await;

        if result.is_err() {
            let _ = fs::remove_file(&part);
        }

        result
    }

    pub fn delete(&self, model_id: &str) -> AppResult<Vec<ModelInfo>> {
        let path = self.model_path(model_id)?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        let part = self.model_path(model_id)?.with_extension("bin.part");
        if part.exists() {
            fs::remove_file(part)?;
        }
        if let Some(legacy_path) = self.legacy_model_path(model_id)? {
            if legacy_path.exists() {
                fs::remove_file(legacy_path)?;
            }
            let legacy_part = self
                .legacy_model_path(model_id)?
                .map(|path| path.with_extension("bin.part"));
            if let Some(legacy_part) = legacy_part {
                if legacy_part.exists() {
                    fs::remove_file(legacy_part)?;
                }
            }
        }
        Ok(self.list())
    }

    fn to_info(&self, definition: &ModelDefinition) -> ModelInfo {
        let path = self.installed_path_for_definition(definition);
        let installed = path.is_some();

        ModelInfo {
            id: definition.id.to_string(),
            display_name: definition.display_name.to_string(),
            file_name: definition.file_name.to_string(),
            size_bytes: definition.size_bytes,
            disk_size: definition.disk_size.to_string(),
            language_mode: definition.language_mode.to_string(),
            checksum_algorithm: definition.checksum_algorithm,
            checksum: definition.checksum.to_string(),
            installed,
            path,
        }
    }

    fn installed_path_for_definition(&self, definition: &ModelDefinition) -> Option<PathBuf> {
        let new_path = self.models_dir.join(definition.file_name);
        if new_path.exists()
            && verify_file(
                &new_path,
                definition.checksum_algorithm,
                definition.checksum,
            )
            .is_ok()
        {
            return Some(new_path);
        }

        let legacy_path = self
            .legacy_models_dir
            .as_ref()
            .map(|dir| dir.join(definition.file_name))?;
        if legacy_path.exists()
            && verify_file(
                &legacy_path,
                definition.checksum_algorithm,
                definition.checksum,
            )
            .is_ok()
        {
            return Some(legacy_path);
        }

        None
    }

    fn legacy_model_path(&self, model_id: &str) -> AppResult<Option<PathBuf>> {
        let definition = find_model(model_id)?;
        Ok(self
            .legacy_models_dir
            .as_ref()
            .map(|dir| dir.join(definition.file_name)))
    }
}

pub fn verify_file(path: &Path, algorithm: ChecksumAlgorithm, expected: &str) -> AppResult<()> {
    let bytes = fs::read(path)?;
    let actual = match algorithm {
        ChecksumAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            hasher.update(bytes);
            format!("{:x}", hasher.finalize())
        }
        ChecksumAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(bytes);
            format!("{:x}", hasher.finalize())
        }
    };

    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(AppError::Model(format!(
            "checksum mismatch for {}: expected {expected}, got {actual}",
            path.display()
        )))
    }
}

fn find_model(model_id: &str) -> AppResult<&'static ModelDefinition> {
    MODELS
        .iter()
        .find(|model| model.id == model_id)
        .ok_or_else(|| AppError::Model(format!("unknown model: {model_id}")))
}

#[cfg(test)]
mod tests {
    use super::{verify_file, ChecksumAlgorithm, ModelManager};
    use sha1::{Digest, Sha1};
    use std::fs;

    #[test]
    fn verifies_checksum() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model.bin");
        fs::write(&path, b"voxta").unwrap();
        let mut hasher = Sha1::new();
        hasher.update(b"voxta");
        let checksum = format!("{:x}", hasher.finalize());
        verify_file(&path, ChecksumAlgorithm::Sha1, &checksum).unwrap();
        assert!(verify_file(&path, ChecksumAlgorithm::Sha1, "bad").is_err());
    }

    #[test]
    fn lists_install_state_from_model_directory() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ModelManager::from_dir(dir.path().to_path_buf());
        let models = manager.list();
        assert!(models.iter().any(|model| model.id == "tiny"));
        assert!(models.iter().all(|model| !model.installed));
    }

    #[test]
    fn recognizes_verified_legacy_model_path() {
        let dir = tempfile::tempdir().unwrap();
        let new_dir = dir.path().join("Voxta").join("models");
        let legacy_dir = dir.path().join(super::LEGACY_APP_DIR).join("models");
        fs::create_dir_all(&legacy_dir).unwrap();
        let legacy_model = legacy_dir.join("ggml-tiny.bin");
        fs::write(&legacy_model, b"legacy").unwrap();
        let mut hasher = Sha1::new();
        hasher.update(b"legacy");
        let checksum = Box::leak(format!("{:x}", hasher.finalize()).into_boxed_str());
        let definition = super::ModelDefinition {
            id: "test",
            display_name: "Test",
            file_name: "ggml-tiny.bin",
            size_bytes: 6,
            disk_size: "6 B",
            language_mode: "test",
            checksum_algorithm: ChecksumAlgorithm::Sha1,
            checksum,
        };
        let manager = ModelManager::from_dirs(new_dir, legacy_dir);
        assert_eq!(
            manager.installed_path_for_definition(&definition),
            Some(legacy_model)
        );
    }
}
