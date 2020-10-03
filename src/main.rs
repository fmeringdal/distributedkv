#![feature(proc_macro_hygiene, decl_macro)]

extern crate actix_cors;
extern crate actix_web;
extern crate base64;
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

#[tokio::main]
async fn main() {
    let t = std::thread::spawn(move || master());
    t.join().unwrap();
}
