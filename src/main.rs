#![feature(proc_macro_hygiene, decl_macro)]

extern crate actix_cors;
extern crate actix_web;
extern crate base64;
extern crate curl;
extern crate md5;
extern crate rand;
extern crate reqwest;
extern crate rocksdb;
extern crate serde;
extern crate serde_json;

#[cfg(test)]
extern crate tempdir;

mod master;
mod shared;

use master::master;

use std::env;

#[tokio::main]
async fn main() {
    match env::var("TYPE") {
        Ok(server_type) if server_type == "master" => {
            let h = std::thread::spawn(|| master());
            h.join().unwrap();
        }
        Ok(_) => println!("Invalid server type, it should either be master"),
        _ => println!("Unable to read type of server from env var"),
    }
}
