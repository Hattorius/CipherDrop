// @generated automatically by Diesel CLI.

diesel::table! {
    files (id) {
        id -> Int4,
        file -> Uuid,
        #[max_length = 96]
        file_name -> Varchar,
        #[max_length = 96]
        file_type -> Varchar,
        #[max_length = 44]
        key -> Varchar,
        #[max_length = 16]
        nonce -> Varchar,
        available_till -> Timestamp,
        date_created -> Timestamp,
    }
}

diesel::table! {
    s3_buckets (id) {
        id -> Int4,
        #[max_length = 64]
        bucket_name -> Varchar,
        #[max_length = 64]
        region -> Varchar,
        #[max_length = 256]
        endpoint -> Varchar,
        #[max_length = 1028]
        access_key -> Varchar,
        #[max_length = 1028]
        secret_key -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(files, s3_buckets,);
