use crate::shared::{key2path, key2volumes};
use actix_cors::Cors;
use actix_web::{delete, get, put, web, App, HttpResponse, HttpServer, Responder};
use rand::seq::SliceRandom;
use reqwest;
use rocksdb::DB;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
struct Meta {
    kvolumes: Vec<String>,
    kpath: String,
}

#[get("/{key}")]
async fn get_key(web::Path(key): web::Path<String>, db: web::Data<DB>) -> impl Responder {
    let bkey = key.as_bytes();
    match db.get(bkey) {
        Ok(Some(value)) => {
            let meta: Meta = serde_json::from_slice(&value).unwrap();
            let remote = format!(
                "http://{}{}",
                meta.kvolumes.choose(&mut rand::thread_rng()).unwrap(),
                meta.kpath
            );
            HttpResponse::TemporaryRedirect()
                .header("Location", remote)
                .finish()
        }
        _ => HttpResponse::NotFound().finish(),
    }
}

#[derive(Serialize, Deserialize)]
struct ListItem {
    key: String,
    kvolumes: Vec<String>,
}

#[delete("/{key}")]
async fn delete_key(
    web::Path(key): web::Path<String>,
    db: web::Data<DB>,
    client: web::Data<reqwest::Client>,
) -> impl Responder {
    let bkey = key.as_bytes();
    match db.get(bkey) {
        Ok(Some(value)) => {
            let meta: Meta = serde_json::from_slice(&value).unwrap();
            for v in meta.kvolumes.iter() {
                let remote = format!("http://{}{}", v, meta.kpath);
                let _ = remote_delete(&remote, &client).await;
            }
            match db.delete(key.as_bytes()) {
                Ok(_) => HttpResponse::NoContent().finish(),
                _ => HttpResponse::InternalServerError().finish(),
            }
        }
        _ => HttpResponse::NotFound().finish(),
    }
}

#[put("/{key}")]
async fn put_key(
    web::Path(key): web::Path<String>,
    bytes: web::Bytes,
    db: web::Data<DB>,
    client: web::Data<reqwest::Client>,
    config: web::Data<AppConfig>,
) -> impl Responder {
    let key = key.as_bytes();
    match db.get(key) {
        Ok(None) => {
            let kvolumes = key2volumes(
                &key.to_vec(),
                &config.volumes,
                config.replicas,
                config.subvolumes,
            );
            let kpath = key2path(&key.to_vec());
            for v in kvolumes.iter() {
                let remote = format!("http://{}{}", v, kpath);
                if let Err(_) = remote_put(&remote, bytes.to_vec(), &client).await {
                    return HttpResponse::InternalServerError().finish();
                };
            }
            let meta = Meta { kvolumes, kpath };
            return match db.put(key, serde_json::to_string(&meta).unwrap()) {
                Ok(_) => HttpResponse::Created().finish(),
                Err(_) => HttpResponse::InternalServerError().finish(),
            };
        }
        _ => HttpResponse::Conflict().finish(),
    }
}

async fn remote_put(
    remote: &String,
    body: Vec<u8>,
    client: &reqwest::Client,
) -> Result<(), reqwest::Error> {
    client.put(remote).body(body).send().await?;
    Ok(())
}

async fn remote_delete(remote: &String, client: &reqwest::Client) -> Result<(), reqwest::Error> {
    client.delete(remote).send().await?;
    Ok(())
}

async fn remote_get(remote: &String) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    client.get(remote).send().await?;
    Ok(())
}

async fn remote_head(remote: &String) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    client.head(remote).send().await?;
    Ok(())
}

struct AppConfig {
    volumes: Vec<String>,
    replicas: usize,
    subvolumes: usize,
}

impl AppConfig {
    pub fn new() -> Self {
        let volumes =
            env::var("VOLUMES").expect("Volume servers was not provided to the master server");
        let volumes: Vec<String> = volumes.split(",").map(|v| String::from(v)).collect();
        let default_replicas = 3;
        let replicas = env::var("REPLICAS")
            .unwrap_or(default_replicas.to_string())
            .parse::<usize>()
            .unwrap_or(default_replicas);
        let default_subvolumes = 2;
        let subvolumes = env::var("SUBVOLUMES")
            .unwrap_or(default_subvolumes.to_string())
            .parse::<usize>()
            .unwrap_or(default_subvolumes);

        if volumes.len() < replicas {
            panic!("There cannot be more replicas then there are volume servers");
        }

        Self {
            volumes,
            replicas,
            subvolumes,
        }
    }
}

#[actix_web::main]
pub async fn master() {
    // Required vars
    let db_path = env::var("DB").unwrap();

    let server_address = env::var("SERVER_ADDRESS").unwrap_or(String::from("127.0.0.1"));
    let server_port = env::var("SERVER_PORT").unwrap_or(String::from("3000"));
    let server_port = server_port.parse::<u16>().unwrap();

    let database = DB::open_default(db_path).unwrap();
    let database = web::Data::new(database);

    let client = reqwest::Client::new();
    let client = web::Data::new(client);

    let config = AppConfig::new();
    let config = web::Data::new(config);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::new().supports_credentials().finish())
            .app_data(database.clone())
            .app_data(client.clone())
            .app_data(config.clone())
            .data(web::PayloadConfig::new(1 << 24))
            .service(get_key)
            .service(put_key)
            .service(delete_key)
    })
    .bind(format!("{}:{}", server_address, server_port))
    .unwrap()
    .run()
    .await
    .unwrap();
}
