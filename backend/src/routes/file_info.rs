use actix_web::{get, web, Error, HttpResponse};

use crate::{
    files::get_file,
    routes::{HttpApiResponse, HttpFileApiResponse},
    DbPool,
};

#[get("/api/file/{file_uuid}")]
async fn file_info(
    pool: web::Data<DbPool>,
    path: web::Path<(String,)>,
) -> Result<HttpResponse, Error> {
    let file_uuid = match uuid::Uuid::try_parse(path.into_inner().0.as_str()) {
        Ok(uuid) => uuid,
        _ => {
            return Ok(HttpResponse::ExpectationFailed().json(HttpApiResponse {
                success: false,
                message: "Invalid UUID".to_string(),
            }))
        }
    };

    let file = match get_file(&pool, file_uuid).await {
        Ok(file) => file,
        _ => {
            return Ok(HttpResponse::InternalServerError().json(HttpApiResponse {
                success: false,
                message: "Internal error, please try again later".to_string(),
            }))
        }
    };

    Ok(HttpResponse::Ok().json(HttpFileApiResponse {
        success: true,
        file_name: file.file_name,
        available_till: file.available_till.and_utc().timestamp(),
    }))
}
