use std::sync::Arc;

use actix_files::Files;
use actix_web::{web, App, HttpServer};
use deadpool::managed::Pool;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use jobs::cleanup_job;
use lazy_static::lazy_static;
use routes::{download_file::download_file, file_html::file_html, upload::upload};
use tera::Tera;

mod crypt;
mod database;
mod files;
mod jobs;
mod routes;
mod s3;
mod schema;

type DbPool = deadpool::managed::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

lazy_static! {
    static ref TEMPLATES: Tera = Tera::new("./../frontend/templates/**/*.html").unwrap();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(conn_spec);
    let pool: DbPool = Pool::builder(config)
        .build()
        .expect("Failed creating database pool");

    let cleanup_pool = pool.clone();
    tokio::spawn(async move {
        let _ = cleanup_job(Arc::new(cleanup_pool)).await;
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(TEMPLATES.clone()))
            .service(upload)
            .service(download_file)
            .service(file_html)
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
