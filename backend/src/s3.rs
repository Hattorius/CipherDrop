use actix_web::web;
use s3::Bucket;

use crate::{actions::get_s3_bucket, DbPool};

pub async fn get_s3_bucket_info(pool: web::Data<DbPool>) -> Option<Box<Bucket>> {
    let bucket_info = match web::block(move || {
        pool.get()
            .ok()
            .and_then(|mut conn| match get_s3_bucket(&mut conn) {
                Ok(maybe_bucket) => maybe_bucket,
                Err(_) => None,
            })
    })
    .await
    {
        Ok(maybe_bucket) => maybe_bucket,
        Err(_) => None,
    };

    let bucket_info = match bucket_info {
        Some(bucket) => bucket,
        None => {
            return None;
        }
    };

    let bucket = s3::bucket::Bucket::new(
        &bucket_info.bucket_name,
        s3::Region::Custom {
            region: bucket_info.region,
            endpoint: bucket_info.endpoint,
        },
        awscreds::Credentials {
            access_key: Some(bucket_info.access_key),
            secret_key: Some(bucket_info.secret_key),
            security_token: None,
            session_token: None,
            expiration: None,
        },
    )
    .unwrap();

    Some(bucket)
}
