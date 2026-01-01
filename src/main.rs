use std::fs::File;

use crate::{db::Db, query::execute, schema::parse_tables};

mod btree;
mod cell;
mod db;
mod header;
mod page;
mod query;
mod schema;
mod value;
mod varint;

fn main() {
    let mut file = match File::open("chinook.db") {
        Ok(f) => f,
        Err(e) => panic!("Failed to open file: {}", e),
    };

    let header = header::parse_header(&mut file);

    let tables = parse_tables(&mut file, header.page_size);

    println!("Tables: {:?}", tables);

    let mut db = Db {
        file,
        page_size: header.page_size,
        tables,
    };

    let rows = execute(&mut db, String::from("SELECT * FROM albums"));

    for row in &rows {
        println!("{:?}", row)
    }
    // execute(&mut db, String::from("SELECT * FROM artists"));
    // execute(&mut db, String::from("SELECT * FROM customers"));
}
