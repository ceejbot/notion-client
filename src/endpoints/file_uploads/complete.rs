use crate::{
    endpoints::NOTION_URI,
    objects::{file_upload::FileUpload, Response},
    NotionClientError,
};

use super::FileUploadsEndpoint;

impl FileUploadsEndpoint {
    /// Complete a multi-part file upload
    ///
    /// This endpoint finalizes a multi-part upload after all parts have been sent.
    /// Only use this for uploads created with mode=multi_part.
    pub async fn complete_file_upload(
        &self,
        file_upload_id: &str,
    ) -> Result<FileUpload, NotionClientError> {
        let result = self
            .client
            .post(format!(
                "{notion_uri}/file_uploads/{file_upload_id}/complete",
                notion_uri = NOTION_URI,
                file_upload_id = file_upload_id
            ))
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
