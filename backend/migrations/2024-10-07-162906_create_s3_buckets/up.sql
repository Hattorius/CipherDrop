-- Your SQL goes here

CREATE TABLE s3_buckets (
  id SERIAL PRIMARY KEY,
  bucket_name VARCHAR(64) NOT NULL,
  region VARCHAR(64) NOT NULL,
  endpoint VARCHAR(256) NOT NULL,
  access_key VARCHAR(1028) NOT NULL,
  secret_key VARCHAR(1028) NOT NULL
)
