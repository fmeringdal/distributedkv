use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};
use rand::Rng;
use rocksdb::{Options, DB};
use serde::{Deserialize, Serialize};
use serde_json::{json, Result, Value};
use std::env;

#[derive(Serialize, Deserialize)]
struct Meta {
    volume: String,
}

#[get("/{key}")]
async fn get_key(web::Path(key): web::Path<String>, db: web::Data<DB>) -> impl Responder {
    match db.get(key.as_bytes()) {
        Ok(Some(value)) => {
            let meta: Meta = serde_json::from_slice(&value).unwrap();
            HttpResponse::TemporaryRedirect()
                .header("Location", format!("{}/{}", meta.volume, key))
                .finish()
        }
        _ => HttpResponse::NotFound().finish(),
    }
}

#[delete("/{key}")]
async fn delete_key(web::Path(key): web::Path<String>, db: web::Data<DB>) -> impl Responder {
    match db.get(key.as_bytes()) {
        Ok(Some(value)) => {
            let meta: Meta = serde_json::from_slice(&value).unwrap();
            HttpResponse::TemporaryRedirect()
                .header("Location", format!("{}/{}", meta.volume, key))
                .finish()
        }
        _ => HttpResponse::NotFound().finish(),
    }
}

#[put("/{key}")]
async fn put_key(web::Path(key): web::Path<String>, db: web::Data<DB>) -> impl Responder {
    match db.get(key.as_bytes()) {
        Ok(None) => {
            if let Ok(volumes) = env::var("VOLUMES") {
                let volumes: Vec<&str> = volumes.split(",").collect();
                let mut rng = rand::thread_rng();
                let volume_index = rng.gen_range(0, volumes.len());
                let volume = volumes[volume_index];

                return HttpResponse::TemporaryRedirect()
                    .header("Location", format!("http://{}/{}", volume, key))
                    .finish();
            }
            HttpResponse::InternalServerError().finish()
        }
        _ => HttpResponse::Conflict().finish(),
    }
}

#[post("/{volume}/{key}")]
async fn post_key(
    web::Path((volume, key)): web::Path<(String, String)>,
    db: web::Data<DB>,
) -> impl Responder {
    println!("Master got a report from volume: {}", volume);
    match db.get(key.as_bytes()) {
        Ok(None) => {
            let meta = Meta {
                volume: format!("http://{}", volume),
            };
            match db.put(key.as_bytes(), serde_json::to_string(&meta).unwrap()) {
                Ok(_) => println!("master hass registered key"),
                Err(_) => println!("coudl not put key in master"),
            }
            HttpResponse::Ok().finish()
        }
        _ => HttpResponse::Conflict().finish(),
    }
}

#[actix_web::main]
pub async fn master() {
    // Required vars
    let volume_servers = env::var("VOLUMES").unwrap();
    let db_path = env::var("DB").unwrap();

    let server_address = env::var("SERVER_ADDRESS").unwrap_or(String::from("127.0.0.1"));
    let server_port = env::var("SERVER_PORT").unwrap_or(String::from("3000"));
    let server_port = server_port.parse::<u16>().unwrap();

    let database = DB::open_default(db_path).unwrap();
    let database = web::Data::new(database);

    HttpServer::new(move || {
        App::new()
            .app_data(database.clone())
            .service(get_key)
            .service(put_key)
            .service(post_key)
            .service(delete_key)
    })
    .bind(format!("{}:{}", server_address, server_port))
    .unwrap()
    .run()
    .await
    .unwrap();
}
