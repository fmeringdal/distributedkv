#![feature(proc_macro_hygiene, decl_macro)]

extern crate actix_cors;
extern crate actix_web;
extern crate base64;
extern crate curl;
extern crate md5;
extern crate rand;
extern crate rocksdb;
extern crate serde;
extern crate serde_json;

mod master;
mod volume;

use master::master;
use volume::volume;

use std::env;

fn main() {
    match env::var("TYPE") {
        Ok(server_type) if server_type == "master" => master(),
        Ok(server_type) if server_type == "volume" => volume(),
        Ok(_) => println!("Invalid server type, it should either be master or volume"),
        _ => println!("Unable to read type of server from env var"),
    }
}
