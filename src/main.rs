use std::fs::File;

use crate::page::Page;

mod database;
mod interior;
mod leaf;
mod page;

// read from chinook.db
// the first 100 bytes are reserved for the header

fn main() {
    let mut file = match File::open("chinook.db") {
        Ok(f) => f,
        Err(e) => panic!("Failed to open file: {}", e),
    };

    let header = database::parse_header(&mut file);

    println!("Page size: {}", header.page_size);

    let page1 = Page::read(&mut file, 47, header.page_size);

    println!("Number of cells: {}", page1.num_cells);

    // we iterate over each cell to get its page number and rowid
    for i in 0..page1.num_cells {
        println!("Cell pointer for {}: {}", i, page1.cell_pointer(i));

        if page1.is_leaf() {
            println!("Leaf cell: {}", i);
            let cell = leaf::parse_cell(page1.cell_pointer(i), &page1.data);
            println!("Values: {:?}", cell.values);
        } else {
            let cell = interior::parse_cell(page1.cell_pointer(i), &page1.data);
            println!("Interior cell: {}", i);
            println!("Rowid (first byte): {}", cell.rowid);
            println!("Child page number: {}", cell.child_page_number);
        }
    }

    if !page1.is_leaf() {
        let rightmost =
            u32::from_be_bytes([page1.data[8], page1.data[9], page1.data[10], page1.data[11]]);
        println!("Right-most child: page {}", rightmost);
    }
}
