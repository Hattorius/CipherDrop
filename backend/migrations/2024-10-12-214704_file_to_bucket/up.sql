-- Your SQL goes here

ALTER TABLE files ADD s3_bucket_id INTEGER NOT NULL REFERENCES s3_buckets(id);
