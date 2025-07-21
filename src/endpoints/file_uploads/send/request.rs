use derive_builder::Builder;

/// Request to send file content to a file upload
#[derive(Debug, Clone, Builder)]
#[builder(setter(strip_option))]
pub struct SendFileUploadRequest {
    /// The filename for the file
    pub filename: String,
    /// The MIME type of the file content
    pub content_type: String,
    /// The raw file data as bytes
    pub file_data: Vec<u8>,
    /// The part number for multi-part uploads (1-indexed)
    /// Leave as None for single-part uploads
    pub part_number: Option<u32>,
}

/// Configuration for streaming multipart uploads
/// 
/// This struct configures how files are uploaded when using streaming upload methods.
/// It allows you to specify the file metadata and control memory usage through chunk sizing.
/// 
/// # Examples
/// 
/// ```no_run
/// use notion_client::endpoints::file_uploads::send::request::StreamingUploadConfig;
/// # use std::io::Error;
/// # async fn example() -> Result<(), Error> {
/// 
/// // Basic configuration with known size
/// let config = StreamingUploadConfig::new(
///     "video.mp4".to_string(),
///     "video/mp4".to_string(),
///     1024 * 1024 * 500, // 500MB
/// );
/// 
/// // Auto-detect MIME type and file size from file path
/// let config = StreamingUploadConfig::from_file_path("document.pdf").await?;
/// 
/// // For streams with unknown size (network, stdin, etc.)
/// let config = StreamingUploadConfig::for_unknown_size(
///     "stream_data.json".to_string(),
///     "application/json".to_string(),
/// );
/// 
/// // Customize chunk size for memory optimization
/// let config = StreamingUploadConfig::new("huge_file.bin".to_string(), "application/octet-stream".to_string(), 1024 * 1024 * 1000)
///     .with_chunk_size(1024 * 1024); // Use 1MB chunks instead of default 5MB
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct StreamingUploadConfig {
    /// The filename for the file
    pub filename: String,
    /// The MIME type of the file content
    pub content_type: String,
    /// The total size of the file in bytes (None if unknown)
    pub total_size: Option<u64>,
    /// Size of each chunk to read from the stream (default: 5MB)
    pub chunk_size: usize,
}

impl StreamingUploadConfig {
    /// Create a new streaming upload configuration with known size
    pub fn new(filename: String, content_type: String, total_size: u64) -> Self {
        Self {
            filename,
            content_type,
            total_size: Some(total_size),
            chunk_size: 5 * 1024 * 1024, // Default to 5MB chunks
        }
    }

    /// Create streaming config from a file path, auto-detecting MIME type and file size
    pub async fn from_file_path<P: AsRef<std::path::Path>>(file_path: P) -> std::io::Result<Self> {
        let path = file_path.as_ref();
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        let content_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        // Auto-detect file size
        let metadata = tokio::fs::metadata(path).await?;
        let total_size = metadata.len();

        Ok(Self {
            filename,
            content_type,
            total_size: Some(total_size),
            chunk_size: 5 * 1024 * 1024,
        })
    }

    /// Create streaming config for unknown-size streams (network, stdin, generated content, etc.)
    /// 
    /// This will always use multi-part upload mode since the total size is unknown.
    pub fn for_unknown_size(filename: String, content_type: String) -> Self {
        Self {
            filename,
            content_type,
            total_size: None,
            chunk_size: 5 * 1024 * 1024, // Default to 5MB chunks
        }
    }

    /// Set a custom chunk size for streaming uploads
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Check if this configuration has a known total size
    pub fn has_known_size(&self) -> bool {
        self.total_size.is_some()
    }

    /// Get the total size if known
    pub fn total_size(&self) -> Option<u64> {
        self.total_size
    }
}

impl SendFileUploadRequest {
    /// Create a new send request for single-part upload
    pub fn single_part(filename: String, content_type: String, file_data: Vec<u8>) -> Self {
        Self {
            filename,
            content_type,
            file_data,
            part_number: None,
        }
    }

    /// Create a new send request for multi-part upload
    pub fn multi_part(
        filename: String,
        content_type: String,
        file_data: Vec<u8>,
        part_number: u32,
    ) -> Self {
        Self {
            filename,
            content_type,
            file_data,
            part_number: Some(part_number),
        }
    }
}
