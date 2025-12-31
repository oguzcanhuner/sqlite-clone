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

    traverse(&mut file, 1, header.page_size);
}

// follow the cell references in interior pages and fetch values from
// linked leaf pages
fn traverse(file: &mut File, page_num: u32, page_size: u16) {
    let page = Page::read(file, page_num, page_size);

    if page.is_leaf() {
        for i in 0..page.num_cells {
            let cell = leaf::parse_cell(page.cell_pointer(i), &page.data);

            // this is only relevant to the sqlite_master table on page 1
            if cell.values[0].as_text() == Some("table") {
                println!(
                    "name: {:?}, root_page: {:?}",
                    cell.values[1].as_text().unwrap(),
                    cell.values[3].as_integer().unwrap()
                );
            }
        }
    } else {
        for i in 0..page.num_cells {
            let cell = interior::parse_cell(page.cell_pointer(i), &page.data);
            traverse(file, cell.child_page_number, page_size);
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
        traverse(file, rightmost, page_size);
    }
}
