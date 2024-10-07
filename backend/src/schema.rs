// @generated automatically by Diesel CLI.

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
