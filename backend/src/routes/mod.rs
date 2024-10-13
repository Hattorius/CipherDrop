use serde::Serialize;

pub mod download_file;
pub mod file_html;
pub mod upload;

#[derive(Serialize)]
struct HttpApiResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct HttpFileUploadApiResponse {
    success: bool,
    uuid: String,
}
