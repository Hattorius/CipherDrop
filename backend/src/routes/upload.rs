use actix_multipart::Multipart;
use actix_web::{post, web, Error, HttpResponse};
use futures_util::StreamExt;
use uuid::Uuid;

use crate::{
    crypt::encrypt,
    files::create_file,
    routes::{HttpApiResponse, HttpFileUploadApiResponse},
    s3, DbPool,
};

const MAX_SIZE: usize = 1_073_741_824; // 1GB in bytes

#[post("/api/upload")]
async fn upload(pool: web::Data<DbPool>, mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut file_name = None;
    let mut file_type = None;
    let mut unique_id = None;
    let mut encrypted_file = None;
    let mut lifetime: Option<i64> = None;

    let bucket = match s3::get_s3_bucket_info(&pool).await {
        Some(bucket) => bucket,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(HttpApiResponse {
                success: false,
                message: "Couldn't receive storage".to_string(),
            }))
        }
    };

    let bucket_id = bucket.id;
    let bucket = bucket.bucket;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition();

        let name = content_disposition
            .and_then(|cd| cd.get_name())
            .unwrap_or("");

        match name {
            "file_name" => {
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    value.extend_from_slice(&data);
                }
                file_name = Some(String::from_utf8(value).unwrap_or_default());
            }
            "file_type" => {
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    value.extend_from_slice(&data);
                }
                file_type = Some(String::from_utf8(value).unwrap_or_default());
            }
            "lifetime" => {
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    value.extend_from_slice(&data);
                }

                let lifetime_str = String::from_utf8(value).unwrap_or_default();
                lifetime = match lifetime_str.as_str() {
                    "1d" => Some(86400),
                    "7d" => Some(86400 * 7),
                    "28d" => Some(86400 * 28),
                    _ => None,
                };
            }
            "file" => {
                let temp_unique_id = Uuid::new_v4();
                let safe_file_name = Some(format!("{}", temp_unique_id));
                let mut value = Vec::new();
                let mut total_size: usize = 0;

                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    total_size += data.len();

                    if total_size > MAX_SIZE {
                        return Ok(HttpResponse::PayloadTooLarge().json(HttpApiResponse {
                            success: false,
                            message: "File size exceeds 1GB".to_string(),
                        }));
                    }

                    value.extend_from_slice(&data);
                }

                let temp_encrypted_file = match encrypt(value) {
                    Some(bytes) => bytes,
                    None => {
                        return Ok(HttpResponse::ServiceUnavailable().json(HttpApiResponse {
                            success: false,
                            message: "Failed encrypting file".to_string(),
                        }))
                    }
                };

                let bucket_result = bucket
                    .put_object(safe_file_name.clone().unwrap(), &temp_encrypted_file.result)
                    .await;
                match bucket_result {
                    Ok(result) => {
                        if result.status_code() != 200 {
                            return Ok(HttpResponse::InternalServerError().json(HttpApiResponse {
                                success: false,
                                message: "File upload errored".to_string(),
                            }));
                        }
                    }
                    Err(e) => {
                        println!("{}", e.to_string());
                        return Ok(HttpResponse::InternalServerError().json(HttpApiResponse {
                            success: false,
                            message: "File upload errored".to_string(),
                        }));
                    }
                }

                encrypted_file = Some(temp_encrypted_file);
                unique_id = Some(temp_unique_id);
            }
            _ => {
                return Ok(HttpResponse::ExpectationFailed().json(HttpApiResponse {
                    success: false,
                    message: "Too many form fields".to_string(),
                }));
            }
        }
    }

    if unique_id.is_none() {
        return Ok(HttpResponse::BadRequest().json(HttpApiResponse {
            success: false,
            message: "Missing form fields".to_string(),
        }));
    }

    if let (
        Some(file_name),
        Some(file_type),
        Some(encrypted_file),
        Some(unique_id),
        Some(lifetime),
    ) = (file_name, file_type, encrypted_file, unique_id, lifetime)
    {
        let result = create_file(
            &pool,
            encrypted_file,
            unique_id,
            file_name,
            file_type,
            lifetime,
            bucket_id,
        )
        .await;

        if result.is_err() {
            let _ = bucket.delete_object(format!("{}", unique_id)).await;
            return Ok(HttpResponse::BadRequest().json(HttpApiResponse {
                success: false,
                message: "Failed saving file".to_string(),
            }));
        }

        return Ok(HttpResponse::Ok().json(HttpFileUploadApiResponse {
            success: true,
            uuid: unique_id.to_string(),
        }));
    } else {
        let _ = bucket
            .delete_object(format!("{}", unique_id.unwrap()))
            .await;
        return Ok(HttpResponse::BadRequest().json(HttpApiResponse {
            success: false,
            message: "Missing form fields".to_string(),
        }));
    }
}
