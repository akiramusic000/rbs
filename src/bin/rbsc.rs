use std::fs;

use rbs::{parse, transpile};

fn main() {
    let str = fs::read_to_string("code.rbs").unwrap();
    let (commands, errors) = parse(str);
    println!("{commands:?}");
    if !errors.is_empty() {
        println!("--------");
        for i in errors {
            println!("{i}");
        }
        println!("--------");
    }
    let py = transpile(commands);
    println!("{py}");
}
