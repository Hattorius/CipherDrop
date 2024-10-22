# Cipher Drop

Goal: Create an anonymous file hosting service.

Available at
- https://cipherdrop.sh
- http://7li2aq2wefmr7ypllk36qyf2ueagvywurhvvmpafadmkgidmgyftetqd.onion

## Features
- Encrypt file on client
- Private key & nonce never leave client machine
- Bytes also get encrypted on the server
- Double encrypted bytes can be saved on any s3

The idea behind this project is to make the file hosting as anonymous as possible. If / when this gets put online it'll have zero logs and you can only see file contents when authorized by the original file uploader.

# How to setup
Make sure you have [docker](https://docs.docker.com/engine/install/) installed.
```
docker compose up
```
Yeah, that's really it. It should now pull the Postgresql image & build the webserver. You should still [setup Diesel & run the migrations](https://github.com/Hattorius/CipherDrop?tab=readme-ov-file#diesel-setup) though!

## Adding s3 buckets
To add your s3 bucket to the database you'll need to attach to the postgres service in docker. First figure out what the postgres container name is:
```
> $ docker ps
CONTAINER ID   IMAGE                COMMAND                  CREATED         STATUS         PORTS                    NAMES
de605f0e7c17   cipherdrop-backend   "backend sh -c ' unt…"   9 minutes ago   Up 3 seconds   0.0.0.0:8080->8080/tcp   cipherdrop
1333910cccd8   postgres:latest      "docker-entrypoint.s…"   9 minutes ago   Up 3 seconds   5432/tcp                 postgres_container
```
This usually is `postgres_container`, but just make sure by checking.

Now run the following command to attach to the container:
```shell
docker exec -it postgres_container psql -U root -d db
```
You can replace `postgres_container` with the name of your postgres container. While being attacked to the container you can run any SQL query. The query to insert a s3 bucket is the following:

```sql
INSERT INTO s3_buckets (bucket_name, region, endpoint, access_key, secret_key)
VALUES ('NAME', 'REGION', 'ENDPOINT', 'ACCESS_KEY', 'SECRET_KEY')
```
With the following example bucket link: `my_bucket.fsn1.your-objectstorage.com` (Hetzner):  
`NAME`: `my_bucket.fsn1`,  
`REGION`: `fsn1`,  
`ENDPOINT`: `your-objectstorage.com`,  
`ACCESS_KEY`: your access key,  
`SECRET_KEY`: your secret key

Why the `NAME` also contains the region? No idea, it's just [how the package I used works.](https://github.com/durch/rust-s3/blob/7c6fdc0646704eac315c11eb60bf9f125975159b/s3/src/bucket.rs#L2548)

# Development setup

This is actually pretty simple, you just have to make sure you have Docker [installed](https://docs.docker.com/desktop/) & running, and run the following command to start a Postgres instance:
```shell
docker compose -f compose-dev.yml up
```

After this you need to copy the [`env.example`](https://github.com/Hattorius/CipherDrop/blob/main/backend/.env) into `.env` in the `backend` folder and change the `DATABASE_URL` value to your database connection string. (which should be `postgres://root:toor@localhost/db`)

## Diesel setup

Make sure to install the [Diesel](https://diesel.rs/guides/getting-started) cli. These are the commands I quickly copied over, but make sure to check if they're not outdated:
```shell
# Linux/MacOS
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.sh | sh

# Windows
powershell -c "irm https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.ps1 | iex"
```  

Run the database migrations using diesel:
```shell
diesel migration run
```  

## Start

Now finally start the server with hot reload:
```shell
cargo watch -w src -w ../frontend -x run
```

# Thank you for reading
