use diesel::prelude::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
#[diesel(table_name = s3_buckets)]
pub struct S3Bucket {
    pub id: i32,
    pub bucket_name: String,
    pub region: String,
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
}
