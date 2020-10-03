use crate::shared::key2volumes;
use actix_cors::Cors;
use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};
use curl::easy::Easy;
use rand::Rng;
use reqwest;
use rocksdb::{Direction, IteratorMode, DB};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::prelude::*;
use std::io::Read;

#[derive(Serialize, Deserialize)]
struct Meta {
    kvolumes: Vec<String>,
}

#[get("/{key}")]
async fn get_key(web::Path(key): web::Path<String>, db: web::Data<DB>) -> impl Responder {
    match db.get(key.as_bytes()) {
        Ok(Some(value)) => {
            let meta: Meta = serde_json::from_slice(&value).unwrap();
            HttpResponse::TemporaryRedirect()
                .header("Location", format!("http://{}/{}", meta.kvolumes[0], key))
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
async fn delete_key(web::Path(key): web::Path<String>, db: web::Data<DB>) -> impl Responder {
    match db.get(key.as_bytes()) {
        Ok(Some(value)) => {
            let meta: Meta = serde_json::from_slice(&value).unwrap();
            for v in meta.kvolumes.iter() {
                let _ = remote_delete(&v).await;
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
) -> impl Responder {
    let key = key.as_bytes();
    match db.get(key) {
        Ok(None) => {
            if let Ok(volumes) = env::var("VOLUMES") {
                let volumes: Vec<String> = volumes.split(",").map(|v| String::from(v)).collect();
                let kvolumes = key2volumes(&key.to_vec(), &volumes, 3, 1);
                for v in kvolumes.iter() {
                    match remote_put(&v, bytes.to_vec()).await {
                        Err(e) => {
                            println!("repliaca failed to write: {:?}", e);
                            return HttpResponse::InternalServerError().finish();
                        }
                        _ => (),
                    };
                }
                let meta = Meta { kvolumes };
                return match db.put(key, serde_json::to_string(&meta).unwrap()) {
                    Ok(_) => HttpResponse::Created().finish(),
                    Err(_) => HttpResponse::InternalServerError().finish(),
                };
            }
            HttpResponse::InternalServerError().finish()
        }
        _ => HttpResponse::Conflict().finish(),
    }
}

//#[post("/{volume}/{key}/{op}")]
//async fn post_key(
//web::Path((volume, key, op)): web::Path<(String, String, String)>,
//db: web::Data<DB>,
//) -> impl Responder {
//let stored = db.get(key.as_bytes());
//match &op[..] {
//"create" if Ok(None) != stored => HttpResponse::Conflict().finish(),
//"create" => {
//let meta = Meta {
//volume: format!("http://{}", volume),
//};
//match db.put(key.as_bytes(), serde_json::to_string(&meta).unwrap()) {
//Ok(_) => HttpResponse::Created().finish(),
//Err(_) => HttpResponse::InternalServerError().finish(),
//}
//}
//"delete" => {
//if !stored.is_ok() || !stored.unwrap().is_some() {
//return HttpResponse::NotFound().finish();
//}
//match db.delete(key.as_bytes()) {
//Ok(_) => HttpResponse::Ok().finish(),
//Err(_) => HttpResponse::InternalServerError().finish(),
//}
//}
//_ => HttpResponse::MethodNotAllowed().finish(),
//}
//}

async fn remote_put(remote: &String, body: Vec<u8>) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let mut remote_scheme = String::from("http://");
    remote_scheme.push_str(remote);
    let remote = remote_scheme;
    let res = client.put(&remote).body(body).send().await?;
    println!("res: {:?}", res.text().await);
    Ok(())
}

async fn remote_delete(remote: &String) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let mut remote_scheme = String::from("http://");
    remote_scheme.push_str(remote);
    let remote = remote_scheme;
    client.delete(&remote).send().await?;
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

#[actix_web::main]
pub async fn master() {
    // Required vars
    let db_path = env::var("DB").unwrap();

    let server_address = env::var("SERVER_ADDRESS").unwrap_or(String::from("127.0.0.1"));
    let server_port = env::var("SERVER_PORT").unwrap_or(String::from("3000"));
    let server_port = server_port.parse::<u16>().unwrap();

    let database = DB::open_default(db_path).unwrap();
    let database = web::Data::new(database);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::new().supports_credentials().finish())
            .app_data(database.clone())
            .data(web::PayloadConfig::new(1 << 60))
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
