pub mod request;

use crate::{
    endpoints::NOTION_URI,
    objects::{file_upload::FileUpload, Response},
    NotionClientError,
};

use self::request::CreateFileUploadRequest;

use super::FileUploadsEndpoint;

impl FileUploadsEndpoint {
    /// Create a file upload
    ///
    /// This endpoint creates a new file upload session. The returned upload URL
    /// can be used to upload the actual file content.
    pub async fn create_file_upload(
        &self,
        request: CreateFileUploadRequest,
    ) -> Result<FileUpload, NotionClientError> {
        let json = serde_json::to_string(&request)
            .map_err(|e| NotionClientError::FailedToSerialize { source: e })?;

        let result = self
            .client
            .post(format!(
                "{notion_uri}/file_uploads",
                notion_uri = NOTION_URI,
            ))
            .body(json)
            .send()
            .await
            .map_err(|e| NotionClientError::FailedToRequest { source: e })?;

        let body = result
            .text()
            .await
            .map_err(|e| NotionClientError::FailedToText { source: e })?;

        let response = serde_json::from_str(&body)
            .map_err(|e| NotionClientError::FailedToDeserialize { source: e, body })?;

        match response {
            Response::Success(r) => Ok(r),
            Response::Error(e) => Err(NotionClientError::InvalidStatusCode { error: e }),
        }
    }
}
