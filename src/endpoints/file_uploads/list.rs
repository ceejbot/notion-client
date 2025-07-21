pub mod response;

use crate::{endpoints::NOTION_URI, objects::Response, NotionClientError};

use self::response::ListFileUploadsResponse;

use super::FileUploadsEndpoint;

impl FileUploadsEndpoint {
    /// List file uploads
    ///
    /// This endpoint returns a paginated list of file uploads for the workspace.
    pub async fn list_file_uploads(
        &self,
        start_cursor: Option<&str>,
        page_size: Option<u32>,
    ) -> Result<ListFileUploadsResponse, NotionClientError> {
        let mut url = format!("{notion_uri}/file_uploads", notion_uri = NOTION_URI);
        let mut query_params = Vec::new();

        if let Some(cursor) = start_cursor {
            query_params.push(format!("start_cursor={}", urlencoding::encode(cursor)));
        }

        if let Some(size) = page_size {
            query_params.push(format!("page_size={}", size));
        }

        if !query_params.is_empty() {
            url.push('?');
            url.push_str(&query_params.join("&"));
        }

        let result = self
            .client
            .get(&url)
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
