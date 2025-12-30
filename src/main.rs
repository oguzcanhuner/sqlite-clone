use std::{fs::File, io::Read, str::from_utf8};
mod database;
mod interior;

// read from chinook.db
// the first 100 bytes are reserved for the header

fn main() {
    let mut file = match File::open("chinook.db") {
        Ok(f) => f,
        Err(e) => panic!("Failed to open file: {}", e),
    };

    let header = parse_header(&mut file);

    // At byte 100, you'll find a b-tree page header:
    // | Offset | Size | Description |
    // |--------|------|-------------|
    // | 100 | 1 | Page type (0x0D = leaf table, 0x05 = interior table, 0x0A = leaf index, 0x02 = interior index) |
    // | 101 | 2 | First freeblock offset (0 if none) |
    // | 103 | 2 | Number of cells on this page |
    // | 105 | 2 | Start of cell content area |
    // | 107 | 1 | Fragmented free bytes |
    // | 108 | 4 | Right-most child pointer (interior pages only) |
    //
    // Cell pointer array starts at:
    // - Offset 108 for leaf pages (8-byte header)
    // - Offset 112 for interior pages (12-byte header)
    //
    // reading the remainder of page 1 (starting from offset 100)
    // page1 needs to be a vector because page_size is not known at compile time,
    // so it needs to be variable length (which vector is)
    let mut page1 = vec![0u8; header.page_size as usize - 100];

    // add the bytes to the page1 vector
    match file.read_exact(&mut page1) {
        Ok(buffer) => buffer,
        Err(e) => panic!("{}", e),
    }

    // bytes 113 and 114 contain a u16 integer (2 bytes) which stores number of cells on this page
    // a cell is just a reference to a page number and a rowid (which helps figure out
    // which ids are in that page)
    let num_of_cells = u16::from_be_bytes([page1[3], page1[4]]);
    println!("Number of cells: {}", num_of_cells);

    // we iterate over each cell to get its page number and rowid
    for i in 0..num_of_cells {
        let cell = interior::get_cell(i, &page1);

        println!("Cell: {}", i);
        println!("Child page number: {}", cell.child_page_number);
        println!("Rowid (first byte): {}", cell.rowid);
    }

    let rightmost = u32::from_be_bytes([page1[8], page1[9], page1[10], page1[11]]);
    println!("Right-most child: page {}", rightmost);
}

// for now, all we care about is page_size
struct Header {
    page_size: u16,
}

// offset 0-16 = magic string "SQLite format 3/000"
// offfset 16-18 = page size in bytes
fn parse_header(file: &mut File) -> Header {
    let mut header = [0u8; 100];

    // &mut means "give read_exact temporary permission to mutate header without
    // becoming the owner". Once read_exact is done with it, the header gets back ownership.
    //
    // - header — pass ownership (you can't use it after)
    // - &header — immutable borrow (you still own it, they can only read)
    // - &mut header — mutable borrow (you still own it, they can read/write)
    //
    // read_exact mutates header in place so there's no need to reassign it
    match file.read_exact(&mut header) {
        Ok(buffer) => buffer,
        Err(e) => panic!("{}", e),
    }

    let page_size = u16::from_be_bytes([header[16], header[17]]);

    println!("Magic string: {}", from_utf8(&header[0..16]).unwrap());
    println!("Page size: {}", page_size);

    Header { page_size }
}
