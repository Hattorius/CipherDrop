use actix_web::web;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    actions::{add_file_record, get_file_record},
    crypt::Encrypted,
    models, DbPool,
};

pub async fn create_file(
    pool: &web::Data<DbPool>,
    file: Encrypted,
    unique_id: Uuid,
    file_name: String,
    file_type: String,
    lifetime: i64,
) -> Result<(), ()> {
    if let Some(mut conn) = pool.get().await.ok() {
        let available_till = Utc::now().timestamp() + lifetime;

        return add_file_record(
            &mut conn,
            file,
            unique_id,
            file_name,
            file_type,
            available_till,
        )
        .await;
    }

    Err(())
}

pub async fn get_file(pool: &web::Data<DbPool>, file_uuid: Uuid) -> Result<models::File, ()> {
    if let Some(mut conn) = pool.get().await.ok() {
        return match get_file_record(&mut conn, file_uuid).await {
            Ok(file) => Ok(file),
            _ => Err(()),
        };
    }

    Err(())
}
