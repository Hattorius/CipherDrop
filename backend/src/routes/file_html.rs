use actix_web::{get, web, Error, HttpResponse};
use tera::{Context, Tera};

use crate::{files::get_file, DbPool};

#[get("/file/{file_uuid}")]
pub async fn file_html(
    path: web::Path<(String,)>,
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
) -> Result<HttpResponse, Error> {
    let mut ctx = Context::new();

    let file_uuid = match uuid::Uuid::try_parse(path.into_inner().0.as_str()) {
        Ok(uuid) => uuid,
        _ => {
            ctx.insert("success", "false");
            ctx.insert("msg", "Invalid UUID");
            let rendered = tmpl.render("file.html", &ctx).map_err(|_| {
                actix_web::error::ErrorInternalServerError("Template rendering error")
            })?;
            return Ok(HttpResponse::Ok().content_type("text/html").body(rendered));
        }
    };

    let file = match get_file(&pool, file_uuid).await {
        Ok(file) => file,
        _ => {
            ctx.insert("success", "false");
            ctx.insert("msg", "Internal error, please try again later");
            let rendered = tmpl.render("file.html", &ctx).map_err(|_| {
                actix_web::error::ErrorInternalServerError("Template rendering error")
            })?;
            return Ok(HttpResponse::Ok().content_type("text/html").body(rendered));
        }
    };

    ctx.insert("success", "true");
    ctx.insert("uuid", &file_uuid.to_string());
    ctx.insert("available_till", &file.available_till.and_utc().timestamp());
    ctx.insert("file_name", &file.file_name);
    let rendered = tmpl
        .render("file.html", &ctx)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Template rendering error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}
