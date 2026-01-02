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

pub use cell::Row;
pub use value::Value;

pub fn run(file_path: &String, query: &String) -> Vec<Row> {
    println!("file_path: {}, query: {}", file_path, query);
    let mut file = File::open(file_path).expect("Failed to open file: {}");

    let header = header::parse_header(&mut file);

    let tables = parse_tables(&mut file, header.page_size);

    println!("Tables: {:?}", tables);

    let mut db = Db {
        file,
        page_size: header.page_size,
        tables,
    };

    execute(&mut db, String::from(query))
}
