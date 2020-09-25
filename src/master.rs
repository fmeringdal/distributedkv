use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};
use rand::Rng;
use rocksdb::{Direction, IteratorMode, Options, DB};
use serde::{Deserialize, Serialize};
use serde_json::{json, Result, Value};
use std::env;

#[derive(Serialize, Deserialize)]
struct Meta {
    volume: String,
}

#[get("/{key}")]
async fn get_key(web::Path(key): web::Path<String>, db: web::Data<DB>) -> impl Responder {
    println!("---------------------------------------------------");
    for (key, value) in db.iterator(IteratorMode::Start) {
        println!(
            "Saw {:?} {:?}",
            String::from_utf8(key.into_vec()).unwrap(),
            String::from_utf8(value.into_vec()).unwrap()
        );
    }

    for (key, value) in db.iterator(IteratorMode::From(key.as_bytes(), Direction::Forward)) {
        println!(
            "Sawhere {:?} {:?}",
            String::from_utf8(key.into_vec()).unwrap(),
            String::from_utf8(value.into_vec()).unwrap()
        );
    }

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

#[derive(Serialize, Deserialize)]
struct ListItem {
    key: String,
    volume: String,
}

#[get("/{key}/list")]
async fn list_key(web::Path(key): web::Path<String>, db: web::Data<DB>) -> impl Responder {
    let list: Vec<ListItem> = db
        .iterator(IteratorMode::From(key.as_bytes(), Direction::Forward))
        .map(|(key, value)| ListItem {
            key: String::from_utf8(key.into_vec()).unwrap(),
            volume: serde_json::from_slice::<Meta>(&value.into_vec())
                .unwrap()
                .volume,
        })
        .collect();

    HttpResponse::Ok().json(list)
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

#[post("/{volume}/{key}/{op}")]
async fn post_key(
    web::Path((volume, key, op)): web::Path<(String, String, String)>,
    db: web::Data<DB>,
) -> impl Responder {
    //let mut iter = db.iterator(IteratorMode::Start);
    //for (key, val) in iter {
    //println!("Key: {:?}, Val: {:?}", key, val);
    //}

    let stored = db.get(key.as_bytes());
    match &op[..] {
        "create" => {
            if Ok(None) != stored {
                return HttpResponse::Conflict().finish();
            }
            let meta = Meta {
                volume: format!("http://{}", volume),
            };
            match db.put(key.as_bytes(), serde_json::to_string(&meta).unwrap()) {
                Ok(_) => HttpResponse::Created().finish(),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        "delete" => {
            if !stored.is_ok() || !stored.unwrap().is_some() {
                return HttpResponse::NotFound().finish();
            }
            match db.delete(key.as_bytes()) {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        _ => HttpResponse::MethodNotAllowed().finish(),
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

    HttpServer::new(move || {
        App::new()
            .app_data(database.clone())
            .service(get_key)
            .service(put_key)
            .service(post_key)
            .service(delete_key)
            .service(list_key)
    })
    .bind(format!("{}:{}", server_address, server_port))
    .unwrap()
    .run()
    .await
    .unwrap();
}
