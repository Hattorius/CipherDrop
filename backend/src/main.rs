use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{post, web, App, Error, HttpResponse, HttpServer};
use crypt::encrypt;
use deadpool::managed::Pool;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use files::create_file;
use futures_util::StreamExt;
use uuid::Uuid;

mod actions;
mod crypt;
mod files;
mod models;
mod s3;
mod schema;

type DbPool = deadpool::managed::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
const MAX_SIZE: usize = 1_073_741_824; // 1GB in bytes

#[post("/upload")]
async fn upload(pool: web::Data<DbPool>, mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut file_name = None;
    let mut file_type = None;
    let mut unique_id = None;
    let mut encrypted_file = None;
    let mut lifetime: Option<i64> = None;

    let bucket = match s3::get_s3_bucket_info(&pool).await {
        Some(bucket) => bucket,
        None => return Ok(HttpResponse::ServiceUnavailable().body("Couldn't receive storage")),
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
                        return Ok(HttpResponse::PayloadTooLarge().body("File size exceeds 1GB"));
                    }

                    value.extend_from_slice(&data);
                }

                let temp_encrypted_file = match encrypt(value) {
                    Some(bytes) => bytes,
                    None => {
                        return Ok(HttpResponse::ServiceUnavailable().body("Failed encrypting file"))
                    }
                };

                let bucket_result = bucket
                    .put_object(safe_file_name.clone().unwrap(), &temp_encrypted_file.result)
                    .await;
                match bucket_result {
                    Ok(result) => {
                        if result.status_code() != 200 {
                            return Ok(
                                HttpResponse::InternalServerError().body("File upload errored")
                            );
                        }
                    }
                    Err(e) => {
                        println!("{}", e.to_string());
                        return Ok(HttpResponse::InternalServerError().body("File upload errored"));
                    }
                }

                encrypted_file = Some(temp_encrypted_file);
                unique_id = Some(temp_unique_id);
            }
            _ => {
                return Ok(HttpResponse::ExpectationFailed().body("Too many form fields"));
            }
        }
    }

    if unique_id.is_none() {
        return Ok(HttpResponse::BadRequest().body("Missing form fields"));
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
            return Ok(HttpResponse::BadRequest().body("Failed saving file"));
        }
    } else {
        let _ = bucket
            .delete_object(format!("{}", unique_id.unwrap()))
            .await;
        return Ok(HttpResponse::BadRequest().body("Missing form fields"));
    }

    Ok(HttpResponse::Ok().body("File uploaded successfully"))
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
