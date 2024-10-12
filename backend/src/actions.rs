use chrono::{Duration, NaiveDateTime, Utc};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    crypt::Encrypted,
    models::{self, NewFile},
    schema::{
        files::{self, available_till, file},
        s3_buckets,
    },
};

type DbError = Box<dyn std::error::Error + Send + Sync>;

pub async fn get_s3_bucket(
    conn: &mut AsyncPgConnection,
) -> Result<Option<models::S3Bucket>, DbError> {
    Ok(s3_buckets::table.first::<models::S3Bucket>(conn).await.ok())
}

pub async fn get_s3_bucket_by_id(
    conn: &mut AsyncPgConnection,
    id: i32,
) -> Result<models::S3Bucket, DbError> {
    Ok(s3_buckets::table
        .filter(s3_buckets::id.eq(id))
        .first::<models::S3Bucket>(conn)
        .await?)
}

pub async fn add_file_record(
    conn: &mut AsyncPgConnection,
    encrypted_file: Encrypted,
    unique_id: Uuid,
    file_name: String,
    file_type: String,
    lifetime: i64,
    s3_bucket_id: i32,
) -> Result<(), ()> {
    let new_file = NewFile {
        file: &unique_id,
        file_name: &file_name,
        file_type: &file_type,
        key: &encrypted_file.key,
        nonce: &encrypted_file.nonce,
        available_till: NaiveDateTime::from_timestamp(lifetime.into(), 0),
        s3_bucket_id: &s3_bucket_id,
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

pub async fn get_file_record(
    conn: &mut AsyncPgConnection,
    file_uuid: Uuid,
) -> Result<models::File, DbError> {
    let found_file = files::table
        .filter(file.eq(file_uuid))
        .first::<models::File>(conn)
        .await?;

    let new_available_till = Utc::now().naive_utc() + Duration::hours(24);
    if found_file.available_till < new_available_till {
        let _ = diesel::update(files::table.filter(file.eq(file_uuid)))
            .set(available_till.eq(new_available_till))
            .execute(conn)
            .await?;
    }

    Ok(found_file)
}
