use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::{get, web, Error};

#[get("/file/{file_uuid}")]
pub async fn file_html(_path: web::Path<(String,)>) -> Result<NamedFile, Error> {
    let path: PathBuf = PathBuf::from("./../frontend/file.html");
    NamedFile::open(path).map_err(|_| actix_web::error::ErrorNotFound("File not found"))
}
