use actix_files::Files;
use actix_web::{web, App, HttpServer};
use deadpool::managed::Pool;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use routes::{
    download_file::download_file, file_html::file_html, file_info::file_info, upload::upload,
};

mod actions;
mod crypt;
mod files;
mod models;
mod routes;
mod s3;
mod schema;

type DbPool = deadpool::managed::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

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
