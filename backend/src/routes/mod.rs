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
struct HttpFileApiResponse {
    success: bool,
    file_name: String,
    available_till: i64,
}

#[derive(Serialize)]
struct HttpFileUploadApiResponse {
    success: bool,
    uuid: String,
}
