extern crate leveldb;
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

    if server_type == "master" {
        master();
    } else if server_type == "volume" {
        volume();
    }
}
