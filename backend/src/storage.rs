use std::num::TryFromIntError;

use s3::{creds::Credentials, Bucket, Region};
use thiserror::Error;
use uuid::Uuid;

use crate::config::{ObjectStorageSettings, Settings};

#[derive(Clone)]
pub struct StorageClient {
    bucket_client: Box<Bucket>,
    bucket: String,
    prefix: String,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("object storage is not configured")]
    MissingConfig,
    #[error("upload is too large")]
    UploadTooLarge,
    #[error("object storage credentials are invalid: {0}")]
    Credentials(String),
    #[error("object storage bucket configuration is invalid: {0}")]
    Bucket(String),
    #[error("failed to store object: {0}")]
    PutObject(String),
}

impl StorageClient {
    pub fn from_settings(settings: &Settings) -> Option<Self> {
        settings
            .object_storage
            .as_ref()
            .and_then(|settings| match Self::from_object_storage(settings) {
                Ok(client) => Some(client),
                Err(error) => {
                    tracing::error!(%error, "object storage configuration is invalid");
                    None
                }
            })
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn dataset_key(&self, owner_sub: &str, dataset_id: Uuid, filename: &str) -> String {
        let logical_key = format!(
            "datasets/{}/{}/{}",
            sanitize_key_segment(owner_sub),
            dataset_id,
            sanitize_key_segment(filename)
        );

        self.key_with_prefix(&logical_key)
    }

    pub async fn put_csv(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), StorageError> {
        let content_length = i64::try_from(bytes.len())
            .map_err(|_: TryFromIntError| StorageError::UploadTooLarge)?;

        tracing::debug!(key, content_length, "uploading CSV object");

        self.bucket_client
            .put_object_with_content_type(key, bytes.as_slice(), content_type)
            .await
            .map_err(|error| StorageError::PutObject(error.to_string()))?;

        Ok(())
    }

    fn from_object_storage(settings: &ObjectStorageSettings) -> Result<Self, StorageError> {
        let credentials = Credentials::new(
            Some(settings.access_key_id.as_str()),
            Some(settings.secret_access_key.as_str()),
            None,
            None,
            None,
        )
        .map_err(|error| StorageError::Credentials(error.to_string()))?;
        let region = Region::Custom {
            region: settings.region.clone(),
            endpoint: settings.endpoint_url.clone(),
        };
        let bucket_client = Bucket::new(settings.bucket.as_str(), region, credentials)
            .map_err(|error| StorageError::Bucket(error.to_string()))?
            .with_path_style();

        Ok(Self {
            bucket_client,
            bucket: settings.bucket.clone(),
            prefix: settings.prefix.clone(),
        })
    }

    fn key_with_prefix(&self, logical_key: &str) -> String {
        let logical_key = logical_key.trim_start_matches('/');
        let prefix = self.prefix.trim_matches('/');

        if prefix.is_empty() {
            logical_key.to_owned()
        } else {
            format!("{prefix}/{logical_key}")
        }
    }
}

fn sanitize_key_segment(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect();

    let trimmed = sanitized.trim_matches('-');
    if trimmed.is_empty() {
        "unnamed".to_owned()
    } else {
        trimmed.to_owned()
    }
}
