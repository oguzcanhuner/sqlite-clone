use std::fs::File;

use crate::schema::parse_tables;

mod btree;
mod cell;
mod header;
mod page;
mod schema;
mod value;
mod varint;

fn main() {
    let mut file = match File::open("chinook.db") {
        Ok(f) => f,
        Err(e) => panic!("Failed to open file: {}", e),
    };

    let tables = parse_tables(&mut file);

    for table in &tables {
        println!("{:?}", table);
    }
}
