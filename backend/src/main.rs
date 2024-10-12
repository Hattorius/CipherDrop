use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{get, post, web, App, Error, HttpResponse, HttpServer};
use crypt::{decrypt, encrypt, Encrypted};
use deadpool::managed::Pool;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use files::{create_file, get_file};
use futures_util::StreamExt;
use serde::Serialize;
use uuid::Uuid;

mod actions;
mod crypt;
mod files;
mod models;
mod s3;
mod schema;

type DbPool = deadpool::managed::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
const MAX_SIZE: usize = 1_073_741_824; // 1GB in bytes

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

#[get("/api/file/{file_uuid}/download")]
async fn download_file(
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

    let bucket = match s3::get_s3_bucket_info(&pool).await {
        Some(bucket) => bucket,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(HttpApiResponse {
                success: false,
                message: "Couldn't receive storage".to_string(),
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(conn_spec);
    let pool: DbPool = Pool::builder(config)
        .build()
        .expect("Failed creating database pool");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(upload)
            .service(file_info)
            .service(download_file)
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
