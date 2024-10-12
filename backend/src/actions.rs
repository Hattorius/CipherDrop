use chrono::NaiveDateTime;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    crypt::Encrypted,
    models::{NewFile, S3Bucket},
    schema::{files, s3_buckets},
};

type DbError = Box<dyn std::error::Error + Send + Sync>;

pub async fn get_s3_bucket(conn: &mut AsyncPgConnection) -> Result<Option<S3Bucket>, DbError> {
    Ok(s3_buckets::table.first::<S3Bucket>(conn).await.ok())
}

pub async fn add_file_record(
    conn: &mut AsyncPgConnection,
    file: Encrypted,
    unique_id: Uuid,
    file_name: String,
    file_type: String,
    lifetime: i64,
) -> Result<(), ()> {
    let new_file = NewFile {
        file: &unique_id,
        file_name: &file_name,
        file_type: &file_type,
        key: &file.key,
        nonce: &file.nonce,
        available_till: NaiveDateTime::from_timestamp(lifetime.into(), 0),
    };

    let result = diesel::insert_into(files::table)
        .values(&new_file)
        .execute(conn)
        .await;

    if result.is_err() {
        return Err(());
    }
    Ok(())
}
