-- Your SQL goes here

CREATE TABLE files (
    id SERIAL PRIMARY KEY,
    file UUID NOT NULL,
    file_name VARCHAR(96) NOT NULL,
    file_type VARCHAR(96) NOT NULL,
    key VARCHAR(44) NOT NULL,
    nonce VARCHAR(16) NOT NULL,
    available_till TIMESTAMP NOT NULL,
    date_created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
