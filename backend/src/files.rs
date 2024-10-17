use chrono::Utc;
use diesel_async::AsyncPgConnection;
use uuid::Uuid;

use crate::{
    crypt::Encrypted,
    database::{
        actions::{self, add_file_record, get_file_record},
        models,
    },
    s3::get_s3_specific_bucket,
};

pub async fn create_file(
    conn: &mut AsyncPgConnection,
    file: Encrypted,
    unique_id: Uuid,
    file_name: String,
    file_type: String,
    lifetime: i64,
    s3_bucket_id: i32,
) -> Result<(), ()> {
    let available_till = Utc::now().timestamp() + lifetime;

    return add_file_record(
        conn,
        file,
        unique_id,
        file_name,
        file_type,
        available_till,
        s3_bucket_id,
    )
    .await;
}

pub async fn get_file(conn: &mut AsyncPgConnection, file_uuid: Uuid) -> Result<models::File, ()> {
    match get_file_record(conn, file_uuid).await {
        Ok(file) => Ok(file),
        _ => Err(()),
    }
}

pub async fn delete_file(conn: &mut AsyncPgConnection, file: &models::File) -> Result<(), ()> {
    let bucket = match get_s3_specific_bucket(conn, file.s3_bucket_id).await {
        Some(bucket) => bucket,
        _ => return Err(()),
    };

    let delete_result = bucket.delete_object(format!("{}", file.file)).await;

    if delete_result.is_ok() {
        let _ = actions::delete_file(conn, file.file).await;
    }

    Ok(())
}
