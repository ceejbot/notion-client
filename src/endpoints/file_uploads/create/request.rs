use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::path::Path;

/// Upload mode for file uploads
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum UploadMode {
    SinglePart,
    MultiPart,
}

impl Default for UploadMode {
    fn default() -> Self {
        UploadMode::SinglePart
    }
}

/// Request to create a file upload
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Default, Builder)]
#[builder(setter(strip_option))]
#[builder(default)]
pub struct CreateFileUploadRequest {
    /// The filename for the file
    pub filename: String,
    /// The MIME type of the file (e.g., "image/png", "application/pdf")
    pub content_type: String,
    /// The size of the file in bytes
    pub content_length: u64,
    /// The upload mode (single_part or multi_part)
    pub mode: UploadMode,
}

impl CreateFileUploadRequest {
    /// Create a new file upload request with all required fields
    pub fn new(
        filename: String,
        content_type: String,
        content_length: u64,
        mode: UploadMode,
    ) -> Self {
        Self {
            filename,
            content_type,
            content_length,
            mode,
        }
    }

    /// Create a file upload request from a file path, automatically detecting MIME type
    pub fn from_file_path<P: AsRef<Path>>(
        file_path: P,
        content_length: u64,
        mode: UploadMode,
    ) -> Self {
        let path = file_path.as_ref();
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        let content_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        Self {
            filename,
            content_type,
            content_length,
            mode,
        }
    }

    /// Create a single-part upload request
    pub fn single_part(filename: String, content_type: String, content_length: u64) -> Self {
        Self::new(
            filename,
            content_type,
            content_length,
            UploadMode::SinglePart,
        )
    }

    /// Create a multi-part upload request
    pub fn multi_part(filename: String, content_type: String, content_length: u64) -> Self {
        Self::new(
            filename,
            content_type,
            content_length,
            UploadMode::MultiPart,
        )
    }
}
