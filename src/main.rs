#![feature(proc_macro_hygiene, decl_macro)]

extern crate actix_web;
extern crate base64;
extern crate curl;
extern crate md5;
extern crate rocksdb;
extern crate serde;
extern crate serde_json;
extern crate tempdir;

mod master;
mod volume;

use master::master;
use volume::volume;

use std::env;

fn main() {
    let server_type = match env::var("TYPE") {
        Ok(s) => s,
        _ => String::from("invalid"),
    };
    println!("type: {}", server_type);

    if server_type == "master" {
        master();
    } else if server_type == "volume" {
        volume();
    }
}
