use std::{fs::File, io::Read, str::from_utf8};

// read from chinook.db
// the first 100 bytes are reserved for the header
// offset 0-16 = magic string "SQLite format 3/000"
// offfset 16-18 = page size in bytes

fn main() {
    let mut file = match File::open("chinook.db") {
        Ok(f) => f,
        Err(e) => panic!("Failed to open file: {}", e),
    };

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

    println!("Magic string: {}", from_utf8(&header[0..16]).unwrap());
    println!(
        "Page size: {}",
        u16::from_be_bytes([header[16], header[17]])
    );
}
