use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::objects::user::User;

/// Status of a file upload
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileUploadStatus {
    Pending,
    Processing,
    Complete,
    Failed,
}

/// A file upload object in Notion
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct FileUpload {
    /// The ID of the file upload
    pub id: String,
    /// The type of the file upload (always "file_upload")
    pub object: String,
    /// The filename of the uploaded file
    pub filename: String,
    /// The MIME type of the file contents
    pub content_type: String,
    /// The size of the file in bytes
    pub content_length: Option<u64>,
    /// The URL to upload the file content to (for single_part uploads)
    pub upload_url: Option<String>,
    /// When the file expires (for Notion-hosted files)
    pub expiry_time: Option<DateTime<Utc>>,
    /// Creation timestamp
    pub created_time: DateTime<Utc>,
    /// Last edit timestamp
    pub last_edited_time: DateTime<Utc>,
    /// Whether the upload is archived
    pub archived: bool,
    /// User who created the upload
    pub created_by: User,
    /// Current status of the upload
    pub status: FileUploadStatus,
    /// Request ID for debugging
    pub request_id: Option<String>,
}

/// A simplified file upload for listings
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct FileUploadSummary {
    /// The ID of the file upload
    pub id: String,
    /// The type of the file upload
    #[serde(rename = "type")]
    pub file_type: String,
    /// The filename
    pub filename: String,
    /// The size in bytes
    pub size: Option<u64>,
}
