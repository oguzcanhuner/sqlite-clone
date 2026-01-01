use std::fs::File;

use crate::cell::Row;

mod btree;
mod cell;
mod header;
mod page;
mod value;
mod varint;

#[derive(Debug)]
struct Table {
    name: String,
    rootpage: i64,
}

// read from chinook.db
// the first 100 bytes are reserved for the header

fn main() {
    let mut file = match File::open("chinook.db") {
        Ok(f) => f,
        Err(e) => panic!("Failed to open file: {}", e),
    };

    let header = header::parse_header(&mut file);

    let mut sqlite_master_rows: Vec<Row> = vec![];

    // read sqlite_master table
    btree::traverse(&mut file, 1, header.page_size, &mut sqlite_master_rows);

    let mut tables: Vec<Table> = vec![];
    // save the table name and references
    for row in &sqlite_master_rows {
        if row.values[0].as_text().unwrap() == "table" {
            tables.push(Table {
                name: String::from(row.values[1].as_text().unwrap()),
                rootpage: row.values[3].as_integer().unwrap(),
            })
        }
    }

    for table in &tables {
        println!("{:?}", table);
    }
}
