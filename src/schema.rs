use std::fs::File;

use crate::{btree, cell::Row, header};

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub rootpage: i64,
}

pub fn parse_tables(file: &mut File) -> Vec<Table> {
    let header = header::parse_header(file);

    let mut sqlite_master_rows: Vec<Row> = vec![];

    // read sqlite_master table
    btree::traverse(file, 1, header.page_size, &mut sqlite_master_rows);

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

    tables
}
