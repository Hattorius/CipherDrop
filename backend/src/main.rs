use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{post, web, App, Error, HttpResponse, HttpServer};
use diesel::{r2d2, PgConnection};
use futures_util::StreamExt;
use uuid::Uuid;

mod actions;
mod models;
mod s3;
mod schema;

type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;
const MAX_SIZE: usize = 1_073_741_824; // 1GB in bytes

#[post("/upload")]
async fn upload(pool: web::Data<DbPool>, mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut file_name = None;
    let mut file_type = None;
    let mut safe_file_name = None;

    let bucket = match s3::get_s3_bucket_info(pool).await {
        Some(bucket) => bucket,
        None => return Ok(HttpResponse::ServiceUnavailable().body("Couldn't receive storage")),
    };

    println!("4");
    while let Some(item) = payload.next().await {
        println!("4.1");
        let mut field = item?;
        let content_disposition = field.content_disposition();

        let name = content_disposition
            .and_then(|cd| cd.get_name())
            .unwrap_or("");

        println!("4.2 {}", name);
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
            "file" => {
                let unique_id = Uuid::new_v4();
                safe_file_name = Some(format!("{}", unique_id));
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

                let bucket_result = bucket
                    .put_object(safe_file_name.clone().unwrap(), &value)
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
            }
            _ => {
                return Ok(HttpResponse::ExpectationFailed().body("Too many form fields"));
            }
        }
        println!("4.3")
    }

    if file_name.is_none() || file_type.is_none() {
        let _ = bucket.delete_object(safe_file_name.unwrap()).await;

        return Ok(HttpResponse::BadRequest().body("Missing form fields"));
    }

    if safe_file_name.is_none() {
        return Ok(HttpResponse::BadRequest().body("Missing form fields"));
    }

    Ok(HttpResponse::Ok().body("File uploaded successfully"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_spec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file");

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
