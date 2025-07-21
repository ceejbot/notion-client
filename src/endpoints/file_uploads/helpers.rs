use std::path::Path;

use crate::{objects::file_upload::FileUpload, NotionClientError};
use tokio::io::{AsyncRead, AsyncReadExt};

use super::{
    create::request::{CreateFileUploadRequest, UploadMode},
    send::request::{SendFileUploadRequest, StreamingUploadConfig},
    FileUploadsEndpoint,
};

// 20MB threshold for auto-selecting upload mode
const MULTIPART_THRESHOLD: u64 = 20 * 1024 * 1024;
// 5MB chunk size for multi-part uploads
const CHUNK_SIZE: usize = 5 * 1024 * 1024;

impl FileUploadsEndpoint {
    /// Upload a file with automatic mode selection
    ///
    /// Files under 20MB use single_part mode, files 20MB and over use multi_part mode.
    /// This is the recommended method for most use cases.
    pub async fn upload_file_auto<P: AsRef<Path>>(
        &self,
        file_path: P,
        file_data: Vec<u8>,
    ) -> Result<FileUpload, NotionClientError> {
        let content_length = file_data.len() as u64;
        let mode = if content_length >= MULTIPART_THRESHOLD {
            UploadMode::MultiPart
        } else {
            UploadMode::SinglePart
        };

        let request = CreateFileUploadRequest::from_file_path(file_path, content_length, mode);
        self.upload_file_with_request(request, file_data).await
    }

    /// Upload a file using single-part mode
    ///
    /// Use this for smaller files or when you specifically want single-part upload.
    pub async fn upload_file_single_part<P: AsRef<Path>>(
        &self,
        file_path: P,
        file_data: Vec<u8>,
    ) -> Result<FileUpload, NotionClientError> {
        let content_length = file_data.len() as u64;
        let request = CreateFileUploadRequest::from_file_path(
            file_path,
            content_length,
            UploadMode::SinglePart,
        );
        self.upload_file_with_request(request, file_data).await
    }

    /// Upload a file using multi-part mode
    ///
    /// Use this for larger files or when you specifically want multi-part upload.
    /// The file will be split into chunks and uploaded in parts.
    pub async fn upload_file_multi_part<P: AsRef<Path>>(
        &self,
        file_path: P,
        file_data: Vec<u8>,
    ) -> Result<FileUpload, NotionClientError> {
        let content_length = file_data.len() as u64;
        let request = CreateFileUploadRequest::from_file_path(
            file_path,
            content_length,
            UploadMode::MultiPart,
        );
        self.upload_file_with_request(request, file_data).await
    }

    /// Internal method to handle file upload with a prepared request
    async fn upload_file_with_request(
        &self,
        request: CreateFileUploadRequest,
        file_data: Vec<u8>,
    ) -> Result<FileUpload, NotionClientError> {
        // Step 1: Create the file upload
        let mut file_upload = self.create_file_upload(request.clone()).await?;

        match request.mode {
            UploadMode::SinglePart => {
                // Step 2: Send the file in one part
                let send_request = SendFileUploadRequest::single_part(
                    request.filename,
                    request.content_type,
                    file_data,
                );
                self.send_file_upload(&file_upload.id, send_request).await?;
            }
            UploadMode::MultiPart => {
                // Step 2: Send the file in multiple parts
                let chunks: Vec<&[u8]> = file_data.chunks(CHUNK_SIZE).collect();

                for (index, chunk) in chunks.iter().enumerate() {
                    let part_number = (index + 1) as u32; // Parts are 1-indexed
                    let send_request = SendFileUploadRequest::multi_part(
                        request.filename.clone(),
                        request.content_type.clone(),
                        chunk.to_vec(),
                        part_number,
                    );
                    self.send_file_upload(&file_upload.id, send_request).await?;
                }

                // Step 3: Complete the multi-part upload
                file_upload = self.complete_file_upload(&file_upload.id).await?;
            }
        }

        Ok(file_upload)
    }

    /// Upload a file using a streaming reader with automatic mode selection
    ///
    /// This method reads from the provided AsyncRead stream in chunks, automatically
    /// choosing single_part or multi_part based on the total file size (20MB threshold).
    /// Memory-efficient for large files as it only keeps one chunk in memory at a time.
    ///
    /// # Examples
    /// 
    /// ```no_run
    /// use tokio::fs::File;
    /// use notion_client::endpoints::file_uploads::send::request::StreamingUploadConfig;
    /// # use notion_client::endpoints::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("token".to_string(), None)?;
    ///
    /// // Upload from file
    /// let file = File::open("large_video.mp4").await?;
    /// let config = StreamingUploadConfig::from_file_path("large_video.mp4").await?;
    /// 
    /// let upload = client.file_uploads.upload_file_auto_stream(file, config).await?;
    /// println!("Uploaded: {}", upload.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_file_auto_stream<R: AsyncRead + Unpin>(
        &self,
        reader: R,
        config: StreamingUploadConfig,
    ) -> Result<FileUpload, NotionClientError> {
        let mode = match config.total_size {
            Some(size) if size >= MULTIPART_THRESHOLD => UploadMode::MultiPart,
            Some(_) => UploadMode::SinglePart,
            None => UploadMode::MultiPart, // Unknown size always uses multipart
        };

        let total_size = config.total_size.unwrap_or(0); // We'll update this after reading for unknown sizes
        let request = CreateFileUploadRequest::new(
            config.filename.clone(),
            config.content_type.clone(),
            total_size,
            mode,
        );

        self.upload_file_with_stream(reader, request, config).await
    }

    /// Upload a file using a streaming reader with single-part mode
    ///
    /// For small files or when you specifically want single-part upload.
    /// Note: Single-part uploads still need to read the entire stream into memory.
    pub async fn upload_file_single_part_stream<R: AsyncRead + Unpin>(
        &self,
        reader: R,
        config: StreamingUploadConfig,
    ) -> Result<FileUpload, NotionClientError> {
        // Single-part uploads require known size - if unknown, we need to read everything first
        let total_size = match config.total_size {
            Some(size) => size,
            None => return Err(NotionClientError::IoError { 
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput, 
                    "Single-part uploads require known file size. Use upload_file_auto_stream() or upload_file_multi_part_stream() for unknown-size streams."
                )
            }),
        };

        let request = CreateFileUploadRequest::new(
            config.filename.clone(),
            config.content_type.clone(),
            total_size,
            UploadMode::SinglePart,
        );

        self.upload_file_with_stream(reader, request, config).await
    }

    /// Upload a file using a streaming reader with multi-part mode
    ///
    /// Memory-efficient for large files. Reads and uploads the file in chunks,
    /// keeping only one chunk in memory at a time.
    pub async fn upload_file_multi_part_stream<R: AsyncRead + Unpin>(
        &self,
        reader: R,
        config: StreamingUploadConfig,
    ) -> Result<FileUpload, NotionClientError> {
        let total_size = config.total_size.unwrap_or(0); // For unknown size, we'll calculate during upload
        let request = CreateFileUploadRequest::new(
            config.filename.clone(),
            config.content_type.clone(),
            total_size,
            UploadMode::MultiPart,
        );

        self.upload_file_with_stream(reader, request, config).await
    }

    /// Upload from a stream with unknown size (network, stdin, generated content, etc.)
    /// 
    /// This method is perfect for streams where you don't know the total size ahead of time.
    /// It will automatically use multi-part upload mode and read the stream in chunks.
    /// Memory-efficient as it only keeps one chunk in memory at a time.
    ///
    /// # Examples
    /// 
    /// ```no_run
    /// use notion_client::endpoints::file_uploads::send::request::StreamingUploadConfig;
    /// # use notion_client::endpoints::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("token".to_string(), None)?;
    ///
    /// // Upload from network stream or any unknown-size source
    /// let network_stream = std::io::Cursor::new(b"streaming data from network");
    /// let config = StreamingUploadConfig::for_unknown_size(
    ///     "streamed_data.json".to_string(),
    ///     "application/json".to_string(),
    /// );
    /// 
    /// let upload = client.file_uploads.upload_stream_unknown_size(network_stream, config).await?;
    /// println!("Uploaded: {}", upload.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_stream_unknown_size<R: AsyncRead + Unpin>(
        &self,
        reader: R,
        config: StreamingUploadConfig,
    ) -> Result<FileUpload, NotionClientError> {
        // Force multi-part mode for unknown-size streams
        let request = CreateFileUploadRequest::new(
            config.filename.clone(),
            config.content_type.clone(),
            0, // Size will be ignored for multi-part
            UploadMode::MultiPart,
        );

        self.upload_file_with_stream(reader, request, config).await
    }

    /// Internal method to handle streaming file upload with a prepared request
    async fn upload_file_with_stream<R: AsyncRead + Unpin>(
        &self,
        mut reader: R,
        request: CreateFileUploadRequest,
        config: StreamingUploadConfig,
    ) -> Result<FileUpload, NotionClientError> {
        // Step 1: Create the file upload
        let mut file_upload = self.create_file_upload(request.clone()).await?;

        match request.mode {
            UploadMode::SinglePart => {
                // For single-part, we need to read the entire stream into memory
                let initial_capacity = config.total_size
                    .map(|size| size as usize)
                    .unwrap_or(1024 * 1024); // Default to 1MB if size unknown
                
                let mut file_data = Vec::with_capacity(initial_capacity);
                reader.read_to_end(&mut file_data).await
                    .map_err(|e| NotionClientError::IoError { source: e })?;

                let send_request = SendFileUploadRequest::single_part(
                    config.filename,
                    config.content_type,
                    file_data,
                );
                self.send_file_upload(&file_upload.id, send_request).await?;
            }
            UploadMode::MultiPart => {
                // For multi-part, read and upload in chunks
                let mut part_number = 1u32;
                let mut buffer = vec![0u8; config.chunk_size];
                
                loop {
                    let bytes_read = reader.read(&mut buffer).await
                        .map_err(|e| NotionClientError::IoError { source: e })?;
                    
                    if bytes_read == 0 {
                        break; // End of stream
                    }

                    // Create a chunk with only the bytes we actually read
                    let chunk = buffer[..bytes_read].to_vec();
                    
                    let send_request = SendFileUploadRequest::multi_part(
                        config.filename.clone(),
                        config.content_type.clone(),
                        chunk,
                        part_number,
                    );
                    
                    self.send_file_upload(&file_upload.id, send_request).await?;
                    part_number += 1;
                }

                // Step 3: Complete the multi-part upload
                file_upload = self.complete_file_upload(&file_upload.id).await?;
            }
        }

        Ok(file_upload)
    }
}
