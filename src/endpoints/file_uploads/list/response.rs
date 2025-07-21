use serde::{Deserialize, Serialize};

use crate::objects::file_upload::FileUploadSummary;

/// Response from listing file uploads
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ListFileUploadsResponse {
    /// The type of object returned (always "list")
    #[serde(rename = "type")]
    pub object_type: String,
    /// The list of file uploads
    pub results: Vec<FileUploadSummary>,
    /// Cursor for the next page of results
    pub next_cursor: Option<String>,
    /// Whether there are more results
    pub has_more: bool,
}
