use notion_client::endpoints::Client;
use tokio::fs::File;

const NOTION_TOKEN: &str = ""; // ⚠️ Set your notion token

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let client = Client::new(NOTION_TOKEN.to_string(), None)?;

    // Example 1: upload a small file using the simple API
    let file_data = std::fs::read("examples/small_file.txt")?;
    let _upload = client
        .file_uploads
        .upload_file_auto("examples/small_file.txt", file_data)
        .await?;

    // Example 2: Upload a large file using streaming API (memory-efficient, auto-detect size)
    let file = File::open("examples/large_video.mp4").await?;
    let config = notion_client::endpoints::file_uploads::send::request::StreamingUploadConfig::from_file_path(
        "examples/large_video.mp4"
    ).await?;

    let upload = client
        .file_uploads
        .upload_file_auto_stream(file, config)
        .await?;
    println!("Large file uploaded: id={}", upload.id);

    // Example 3: Upload with custom chunk size for memory optimization
    let file = File::open("examples/data.bin").await?;
    let config = notion_client::endpoints::file_uploads::send::request::StreamingUploadConfig::from_file_path(
        "examples/data.bin"
    ).await?.with_chunk_size(1024 * 1024); // 1MB chunks instead of default 5MB
    let upload = client
        .file_uploads
        .upload_file_multi_part_stream(file, config)
        .await?;
    println!("Custom chunked file uploaded: id={}", upload.id);

    // Example 4: Upload from unknown-size stream (network, stdin, etc.)
    let network_data = b"This could be from a network request, stdin, or any stream with unknown final size.";
    let cursor = std::io::Cursor::new(network_data);

    let config = notion_client::endpoints::file_uploads::send::request::StreamingUploadConfig::for_unknown_size(
        "stream_data.txt".to_string(),
        "text/plain".to_string(),
    );

    let upload = client
        .file_uploads
        .upload_stream_unknown_size(cursor, config)
        .await?;
    println!("Unknown-size stream uploaded: id={}", upload.id);

    Ok(())
}
