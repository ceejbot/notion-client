use crate::{
    endpoints::file_uploads::{
        create::request::{CreateFileUploadRequest, UploadMode},
        list::response::ListFileUploadsResponse,
        send::request::StreamingUploadConfig,
    },
    objects::file_upload::FileUpload,
};

#[test]
fn test_create_file_upload_request() {
    let request = CreateFileUploadRequest::single_part(
        "example.jpg".to_string(),
        "image/jpeg".to_string(),
        1024,
    );

    assert_eq!(request.filename, "example.jpg");
    assert_eq!(request.content_type, "image/jpeg");
    assert_eq!(request.content_length, 1024);
    assert_eq!(request.mode, UploadMode::SinglePart);

    let request_multi = CreateFileUploadRequest::multi_part(
        "document.pdf".to_string(),
        "application/pdf".to_string(),
        2048,
    );

    assert_eq!(request_multi.filename, "document.pdf");
    assert_eq!(request_multi.content_type, "application/pdf");
    assert_eq!(request_multi.content_length, 2048);
    assert_eq!(request_multi.mode, UploadMode::MultiPart);
}

#[test]
fn test_create_file_upload_request_serialization() {
    let request =
        CreateFileUploadRequest::single_part("test.png".to_string(), "image/png".to_string(), 2048);

    let json = serde_json::to_string(&request).expect("Failed to serialize");
    let expected = r#"{"filename":"test.png","content_type":"image/png","content_length":2048,"mode":"single_part"}"#;

    assert_eq!(json, expected);
}

#[test]
fn test_file_upload_deserialization() {
    let json = r#"{
        "id": "12345678-1234-1234-1234-123456789abc",
        "object": "file_upload",
        "filename": "example.jpg",
        "content_type": "image/jpeg",
        "content_length": 1024,
        "upload_url": "https://example.com/upload",
        "expiry_time": "2023-12-01T10:00:00.000Z",
        "created_time": "2023-12-01T09:00:00.000Z",
        "last_edited_time": "2023-12-01T09:00:00.000Z",
        "archived": false,
        "created_by": {"object": "user", "id": "user123", "type": "person"},
        "status": "pending"
    }"#;

    let file_upload: FileUpload = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(file_upload.id, "12345678-1234-1234-1234-123456789abc");
    assert_eq!(file_upload.object, "file_upload");
    assert_eq!(file_upload.filename, "example.jpg");
    assert_eq!(file_upload.content_type, "image/jpeg");
    assert_eq!(file_upload.content_length, Some(1024));
    assert_eq!(
        file_upload.upload_url,
        Some("https://example.com/upload".to_string())
    );
    assert!(file_upload.expiry_time.is_some());
}

#[test]
fn test_list_file_uploads_response_deserialization() {
    let json = r#"{
        "type": "list",
        "results": [
            {
                "id": "12345678-1234-1234-1234-123456789abc",
                "type": "file_upload",
                "filename": "image1.jpg",
                "size": 1024
            },
            {
                "id": "87654321-4321-4321-4321-cba987654321",
                "type": "file_upload",
                "filename": "document.pdf",
                "size": 2048
            }
        ],
        "next_cursor": "cursor123",
        "has_more": true
    }"#;

    let response: ListFileUploadsResponse =
        serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(response.object_type, "list");
    assert_eq!(response.results.len(), 2);
    assert_eq!(response.next_cursor, Some("cursor123".to_string()));
    assert_eq!(response.has_more, true);

    assert_eq!(
        response.results[0].id,
        "12345678-1234-1234-1234-123456789abc"
    );
    assert_eq!(response.results[0].filename, "image1.jpg");
    assert_eq!(response.results[0].size, Some(1024));

    assert_eq!(
        response.results[1].id,
        "87654321-4321-4321-4321-cba987654321"
    );
    assert_eq!(response.results[1].filename, "document.pdf");
    assert_eq!(response.results[1].size, Some(2048));
}

#[test]
fn test_create_file_upload_request_from_json() {
    let json = include_str!("tests/create_file_upload_request.json");
    let request: CreateFileUploadRequest =
        serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(request.filename, "example.jpg");
    assert_eq!(request.content_type, "image/jpeg");
    assert_eq!(request.content_length, 1024);
    assert_eq!(request.mode, UploadMode::SinglePart);
}

#[test]
fn test_create_file_upload_response_from_json() {
    let json = include_str!("tests/create_file_upload_200.json");
    let response: FileUpload = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(response.id, "12345678-1234-1234-1234-123456789abc");
    assert_eq!(response.object, "file_upload");
    assert_eq!(response.filename, "example.png");
    assert_eq!(response.content_type, "image/png");
    assert_eq!(response.content_length, Some(1024));
    assert_eq!(
        response.upload_url,
        Some(
            "https://api.notion.com/v1/file_uploads/12345678-1234-1234-1234-123456789abc/send"
                .to_string()
        )
    );
    assert!(response.expiry_time.is_some());
}

#[test]
fn test_list_file_uploads_response_from_json() {
    let json = include_str!("tests/list_file_uploads_200.json");
    let response: ListFileUploadsResponse =
        serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(response.object_type, "list");
    assert_eq!(response.results.len(), 2);
    assert_eq!(response.next_cursor, Some("cursor123".to_string()));
    assert_eq!(response.has_more, true);
}

#[test]
fn test_retrieve_file_upload_response_from_json() {
    let json = include_str!("tests/retrieve_file_upload_200.json");
    let response: FileUpload = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(response.id, "12345678-1234-1234-1234-123456789abc");
    assert_eq!(response.object, "file_upload");
    assert_eq!(response.filename, "completed_upload.jpg");
    assert!(response.expiry_time.is_some());
}

#[test]
fn test_streaming_upload_config() {
    let config = StreamingUploadConfig::new(
        "test.mp4".to_string(),
        "video/mp4".to_string(),
        1024 * 1024 * 100, // 100MB
    );

    assert_eq!(config.filename, "test.mp4");
    assert_eq!(config.content_type, "video/mp4");
    assert_eq!(config.total_size, Some(1024 * 1024 * 100));
    assert_eq!(config.chunk_size, 5 * 1024 * 1024); // Default 5MB

    let config_with_custom_chunk = config.with_chunk_size(1024 * 1024); // 1MB chunks
    assert_eq!(config_with_custom_chunk.chunk_size, 1024 * 1024);
}

#[test]
fn test_streaming_upload_config_for_unknown_size() {
    let config = StreamingUploadConfig::for_unknown_size(
        "stream_data.json".to_string(),
        "application/json".to_string(),
    );

    assert_eq!(config.filename, "stream_data.json");
    assert_eq!(config.content_type, "application/json");
    assert_eq!(config.total_size, None);
    assert_eq!(config.chunk_size, 5 * 1024 * 1024);
    assert!(!config.has_known_size());
}

#[test]
fn test_streaming_upload_config_helper_methods() {
    let config = StreamingUploadConfig::new(
        "test.bin".to_string(),
        "application/octet-stream".to_string(),
        1024,
    );

    assert!(config.has_known_size());
    assert_eq!(config.total_size(), Some(1024));

    let unknown_config = StreamingUploadConfig::for_unknown_size(
        "unknown.txt".to_string(),
        "text/plain".to_string(),
    );

    assert!(!unknown_config.has_known_size());
    assert_eq!(unknown_config.total_size(), None);
}
