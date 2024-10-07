use diesel::{OptionalExtension, PgConnection, RunQueryDsl};

use crate::models;

type DbError = Box<dyn std::error::Error + Send + Sync>;

pub fn get_s3_bucket(conn: &mut PgConnection) -> Result<Option<models::S3Bucket>, DbError> {
    use crate::schema::s3_buckets::dsl::*;

    let bucket = s3_buckets.first::<models::S3Bucket>(conn).optional()?;
    Ok(bucket)
}
