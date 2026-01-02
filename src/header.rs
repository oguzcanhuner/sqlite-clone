use std::{fs::File, io::Read};

// for now, all we care about is page_size
pub struct Header {
    pub page_size: u16,
}

// offset 0-16 = magic string "SQLite format 3/000"
// offfset 16-18 = page size in bytes
pub fn parse_header(file: &mut File) -> Header {
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

    Header { page_size }
}

// this is just an arbitrary module to group tests in the file. not needed.
#[cfg(test)] // this tells rust to only compile the following code when running tests
mod tests {
    use std::fs::File;

    use crate::header::parse_header;

    #[test]
    // test that parse_header returns a Header with a page size
    fn test_parse_header() {
        let mut file = File::open("tests/chinook.db").unwrap();

        let result = parse_header(&mut file);

        assert_eq!(result.page_size, 1024)
    }
}
