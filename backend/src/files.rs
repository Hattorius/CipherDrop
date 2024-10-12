use actix_web::web;
use chrono::Utc;
use uuid::Uuid;

use crate::{actions::add_file_record, crypt::Encrypted, DbPool};

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
