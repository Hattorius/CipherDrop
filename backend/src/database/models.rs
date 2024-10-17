use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::files;

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

#[derive(Insertable)]
#[diesel(table_name = files)]
pub struct NewFile<'a> {
    pub file: &'a Uuid,
    pub file_name: &'a str,
    pub file_type: &'a str,
    pub key: &'a str,
    pub nonce: &'a str,
    pub available_till: NaiveDateTime,
    pub s3_bucket_id: &'a i32,
}

#[derive(Debug, Clone, Queryable)]
#[diesel(table_name = files)]
pub struct File {
    pub id: i32,
    pub file: uuid::Uuid,
    pub file_name: String,
    pub file_type: String,
    pub key: String,
    pub nonce: String,
    pub available_till: chrono::NaiveDateTime,
    pub date_created: chrono::NaiveDateTime,
    pub s3_bucket_id: i32,
}
