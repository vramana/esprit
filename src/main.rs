extern crate esprit;

use std::fs::File;
use std::io::prelude::*;

fn main() {
    let filename = "tests/adhoc/angular.js".to_string();

    println!("In file {}", filename);

    let mut f = File::open(filename).expect("file not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    // println!("With text:\n{}", contents);

    let p = esprit::script(&contents);

    if let Ok(_) = p {
        println!("successful parse");
    }
}
