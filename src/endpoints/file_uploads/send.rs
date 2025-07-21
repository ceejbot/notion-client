pub mod request;

use crate::{endpoints::NOTION_URI, NotionClientError};

use self::request::SendFileUploadRequest;
use super::FileUploadsEndpoint;

impl FileUploadsEndpoint {
    /// Send file content to a file upload
    ///
    /// This endpoint uploads file content for both single_part and multi_part uploads.
    /// For multi_part uploads, call this multiple times with different part_number values.
    pub async fn send_file_upload(
        &self,
        file_upload_id: &str,
        request: SendFileUploadRequest,
    ) -> Result<(), NotionClientError> {
        let mut form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(request.file_data)
                .file_name(request.filename.clone())
                .mime_str(&request.content_type)
                .map_err(|e| NotionClientError::FailedToBuildRequest { source: e })?,
        );

        // Add part_number for multi-part uploads
        if let Some(part_number) = request.part_number {
            form = form.text("part_number", part_number.to_string());
        }

        let result = self
            .client
            .post(format!(
                "{notion_uri}/file_uploads/{file_upload_id}/send",
                notion_uri = NOTION_URI,
                file_upload_id = file_upload_id
            ))
            .multipart(form)
            .send()
            .await
            .map_err(|e| NotionClientError::FailedToRequest { source: e })?;

        result
            .error_for_status()
            .map_err(|e| NotionClientError::FailedToRequest { source: e })?;

        Ok(())
    }
}
