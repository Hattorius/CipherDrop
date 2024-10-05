use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{post, web, App, Error, HttpResponse, HttpServer};
use futures_util::StreamExt as _;
use std::fs::OpenOptions;
use std::io::Write;
use uuid::Uuid;

const MAX_SIZE: usize = 1_073_741_824; // 1GB in bytes

#[post("/upload")]
async fn upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // Initialize variables for form fields
    let mut file_name = None;
    let mut file_type = None;
    let mut total_size = 0usize;

    // Create the uploads directory if it doesn't exist
    std::fs::create_dir_all("uploads")?;

    // Iterate over multipart stream
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition();

        // Get the field name
        let name = content_disposition
            .and_then(|cd| cd.get_name())
            .unwrap_or("");

        match name {
            "file_name" => {
                // Read the field value
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    value.extend_from_slice(&data);
                }
                file_name = Some(String::from_utf8(value).unwrap_or_default());
            }
            "file_type" => {
                // Read the field value
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    value.extend_from_slice(&data);
                }
                file_type = Some(String::from_utf8(value).unwrap_or_default());
            }
            "file" => {
                // Generate a unique filename
                let unique_id = Uuid::new_v4();
                let safe_file_name = file_name.clone().unwrap_or_else(|| format!("upload_{}", unique_id));
                let filepath = format!("uploads/{}", sanitize_filename::sanitize(&safe_file_name));

                // Open a file to write the uploaded data
                let mut f = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&filepath)?;

                // Process the file field
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    total_size += data.len();

                    // Check file size limit
                    if total_size > MAX_SIZE {
                        // Remove the partially uploaded file
                        std::fs::remove_file(&filepath)?;
                        return Ok(HttpResponse::BadRequest().body("File size exceeds 1GB limit"));
                    }

                    // Write data to file
                    f.write_all(&data)?;
                }
            }
            _ => {
                // Handle unexpected fields if necessary
            }
        }
    }

    // Check if required fields are present
    if file_name.is_none() || file_type.is_none() {
        return Ok(HttpResponse::BadRequest().body("Missing form fields"));
    }

    Ok(HttpResponse::Ok().body("File uploaded successfully"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running at :8080");
    HttpServer::new(|| {
        App::new()
            // Register the upload route first
            .service(upload)
            // Serve static files from the "frontend" directory
            .service(
                Files::new("/", "./../frontend")
                    .index_file("index.html")
                    .use_last_modified(true)
                    .use_etag(true),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
