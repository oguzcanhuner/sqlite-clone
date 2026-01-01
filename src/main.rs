use std::fs::File;

use crate::{leaf::Row, page::Page};

mod database;
mod interior;
mod leaf;
mod page;

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

    let header = database::parse_header(&mut file);

    let mut sqlite_master_rows: Vec<Row> = vec![];

    // read sqlite_master table
    traverse(&mut file, 1, header.page_size, &mut sqlite_master_rows);

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

// follow the cell references in interior pages and fetch values from
// linked leaf pages
fn traverse(file: &mut File, page_num: u32, page_size: u16, records: &mut Vec<Row>) {
    let page = Page::read(file, page_num, page_size);

    if page.is_leaf() {
        for i in 0..page.num_cells {
            let row = leaf::parse_cell(page.cell_pointer(i), &page.data);

            records.push(row);
        }
    } else {
        for i in 0..page.num_cells {
            let cell = interior::parse_cell(page.cell_pointer(i), &page.data);

            traverse(file, cell.child_page_number, page_size, records);
        }

        let mut offset = 0;

        if page_num == 1 {
            offset = 100;
        }

        let rightmost = u32::from_be_bytes([
            page.data[offset + 8],
            page.data[offset + 9],
            page.data[offset + 10],
            page.data[offset + 11],
        ]);
        traverse(file, rightmost, page_size, records);
    }
}
