use actix_web::{get, web, Error, HttpResponse};

use crate::{
    crypt::{decrypt, Encrypted},
    files::get_file,
    routes::HttpApiResponse,
    s3, DbPool,
};

#[get("/api/file/{file_uuid}/download")]
pub async fn download_file(
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

    let bucket = match s3::get_s3_specific_bucket(&pool, file.s3_bucket_id).await {
        Some(bucket) => bucket,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(HttpApiResponse {
                success: false,
                message: "Couldn't receive storage".to_string(),
            }))
        }
    };

    let bytes = match bucket.get_object(format!("{}", file_uuid)).await {
        Ok(file) => file,
        _ => {
            return Ok(HttpResponse::ServiceUnavailable().json(HttpApiResponse {
                success: false,
                message: "Couldn't find file".to_string(),
            }))
        }
    }
    .to_vec();

    let encrypted_file = Encrypted {
        key: file.key,
        nonce: file.nonce,
        result: bytes,
    };

    let file = match decrypt(encrypted_file) {
        Some(file) => file,
        _ => {
            return Ok(HttpResponse::ServiceUnavailable().json(HttpApiResponse {
                success: false,
                message: "Couldn't decrypt file".to_string(),
            }))
        }
    };

    return Ok(HttpResponse::Ok().body(file));
}
