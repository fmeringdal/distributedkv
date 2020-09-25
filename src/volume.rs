use actix_web::{
    delete, get, post, put, web,
    web::{Bytes, Query},
    App, HttpResponse, HttpServer, Responder,
};
use curl::easy::Easy;
use md5;
use std::env;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::Read;
use std::path::PathBuf;
use std::vec::Vec;

struct FileCache {
    pub basedir: String,
    pub tmpdir: String,
}

impl FileCache {
    pub fn new(basedir: String) -> Self {
        let basedir = PathBuf::from(&basedir);
        if !basedir.exists() {
            fs::create_dir_all(&basedir).unwrap();
        }

        let basedir = fs::canonicalize(basedir).unwrap();
        let tmpdir = basedir.clone().join("tmp");
        if tmpdir.exists() {
            fs::remove_dir_all(&tmpdir).unwrap();
        }
        fs::create_dir_all(&tmpdir).unwrap();
        Self {
            basedir: String::from(basedir.to_str().unwrap()),
            tmpdir: String::from(tmpdir.to_str().unwrap()),
        }
    }

    fn k2p(&self, key: &String, mkdir_ok: bool) -> String {
        //key = hashlib.md5(key.encode('utf-8')).hexdigest()
        let digest = md5::compute(key);

        // 2 byte layers deep, meaning a fanout of 256
        // optimized for 2^24 = 16M files per volume server
        // path = self.basedir+"/"+key[0:2]+"/"+key[0:4]
        let path = self.basedir.clone() + "/" + &key[0..2] + "/" + &key[0..4];
        //if not os.path.isdir(path) and mkdir_ok:
        //# exist ok is fine, could be a race
        //os.makedirs(path, exist_ok=True)
        let path_buf = PathBuf::from(&path);
        if !path_buf.exists() {
            fs::create_dir_all(&path_buf).unwrap();
        }

        //return os.path.join(path, key)
        String::from(path_buf.join(key).to_str().unwrap())
    }

    pub fn exists(&self, key: String) -> bool {
        PathBuf::from(self.k2p(&key, true)).exists()
    }

    pub fn delete(&self, key: &String) -> bool {
        let path_buf = PathBuf::from(self.k2p(&key, true));
        if path_buf.exists() {
            fs::remove_file(path_buf).unwrap();
            return true;
        }
        return false;
    }

    pub fn get(&self, key: String) -> Vec<u8> {
        let path_buf = PathBuf::from(self.k2p(&key, true));
        if path_buf.exists() {
            return fs::read(path_buf).unwrap();
        }
        return Vec::default();
    }

    pub fn put(&self, key: &String, stream: Vec<u8>) -> bool {
        let path_buf = PathBuf::from(self.k2p(&key, true));
        println!("Path: {:?}", path_buf);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path_buf)
            .unwrap();
        file.write(&stream).unwrap();
        return false;
    }
}

pub fn report_to_master(key: &String, op: &str) {
    let mut easy = Easy::new();
    let mut data = "".as_bytes();
    easy.url(&format!(
        "{}/{}/{}/{}",
        "http://127.0.0.1:3000",
        get_host(),
        key,
        op
    ))
    .unwrap();
    easy.post(true).unwrap();
    let mut transfer = easy.transfer();
    transfer
        .read_function(|buf| Ok(data.read(buf).unwrap_or(0)))
        .unwrap();
    match transfer.perform() {
        Ok(res) => println!("Success: {:?}", res),
        Err(e) => println!("Error: {:?}", e),
    }
}

#[get("/{key}")]
async fn get_key(web::Path(key): web::Path<String>, fc: web::Data<FileCache>) -> impl Responder {
    HttpResponse::Ok().body(fc.get(key))
}

#[delete("/{key}")]
async fn delete_key(web::Path(key): web::Path<String>, fc: web::Data<FileCache>) -> impl Responder {
    fc.delete(&key);
    report_to_master(&key, "delete");
    HttpResponse::NoContent().finish()
}

#[put("/{key}")]
async fn put_key(
    web::Path(key): web::Path<String>,
    bytes: Bytes,
    fc: web::Data<FileCache>,
) -> impl Responder {
    fc.put(&key, bytes.to_vec());
    report_to_master(&key, "create");
    HttpResponse::Created()
}

pub fn get_host() -> String {
    let server_address = env::var("SERVER_ADDRESS").unwrap_or(String::from("127.0.0.1"));
    let server_port = env::var("SERVER_PORT").unwrap_or(String::from("3000"));
    let server_port = server_port.parse::<u16>().unwrap();
    format!("{}:{}", server_address, server_port)
}

#[actix_web::main]
pub async fn volume() {
    let volume = env::var("VOLUME").unwrap();
    let fc = FileCache::new(volume);
    let fc = web::Data::new(fc);

    HttpServer::new(move || {
        App::new()
            .app_data(fc.clone())
            .service(get_key)
            .service(put_key)
            .service(delete_key)
    })
    .bind(get_host())
    .unwrap()
    .run()
    .await
    .unwrap();
}
