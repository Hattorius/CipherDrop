use actix_web::web;
use s3::Bucket;

use crate::{
    actions::{get_s3_bucket, get_s3_bucket_by_id},
    DbPool,
};

pub struct S3Bucket {
    pub id: i32,
    pub bucket: Box<Bucket>,
}

pub async fn get_s3_bucket_info(pool: &web::Data<DbPool>) -> Option<S3Bucket> {
    let bucket_info = if let Some(mut conn) = pool.get().await.ok() {
        match get_s3_bucket(&mut conn).await {
            Ok(maybe_bucket) => maybe_bucket,
            Err(_) => None,
        }
    } else {
        None
    };

    let bucket_info = match bucket_info {
        Some(bucket) => bucket,
        None => {
            return None;
        }
    };

    let bucket: Box<Bucket> = s3::bucket::Bucket::new(
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

    Some(S3Bucket {
        id: bucket_info.id,
        bucket,
    })
}

pub async fn get_s3_specific_bucket(pool: &web::Data<DbPool>, id: i32) -> Option<Box<Bucket>> {
    let bucket_info = if let Some(mut conn) = pool.get().await.ok() {
        match get_s3_bucket_by_id(&mut conn, id).await {
            Ok(maybe_bucket) => Some(maybe_bucket),
            Err(_) => return None,
        }
    } else {
        return None;
    };

    if let Some(bucket_info) = bucket_info {
        let bucket: Box<Bucket> = s3::bucket::Bucket::new(
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
        return Some(bucket);
    }

    None
}
