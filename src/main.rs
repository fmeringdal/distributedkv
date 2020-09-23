extern crate leveldb;
extern crate tempdir;

mod master;
mod volume;

use master::master;
use volume::volume;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let server_type = match env::var("TYPE") {
        Ok(s) => s,
        _ => String::from("invalid"),
    };
    println!("type:  {}", server_type);
    //master();
    //volume();
    //volume();
    //volume();
}
