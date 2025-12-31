use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

pub struct Page {
    pub data: Vec<u8>,
    pub page_type: u8,
    pub num_cells: u16,
    pub offset: usize,
}

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

impl Page {
    pub fn read(file: &mut File, page_num: u32, page_size: u16) -> Page {
        let mut page = vec![0u8; page_size as usize];

        // go back to the start of the page
        let offset = (page_num - 1) as u64 * page_size as u64;
        file.seek(SeekFrom::Start(offset)).unwrap();

        // read only the bytes of the page
        match file.read_exact(&mut page) {
            Ok(b) => b,
            Err(e) => panic!("{}", e),
        }

        let mut offset: usize = 0;

        // adjust for the 100 byte header on the first page.
        // b-tree header starts after that.
        if page_num == 1 {
            offset = 100
        }

        let num_cells = u16::from_be_bytes([page[offset + 3], page[offset + 4]]);
        let page_type = page[offset];

        Page {
            data: page,
            page_type,
            num_cells,
            offset,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.page_type == 0x0D || self.page_type == 0x0A
    }

    pub fn cell_pointer(&self, i: u16) -> usize {
        // how do you get a cell pointer?
        // there's a cell pointer array which starts at offset 8 (leaf) or 12 (interior)
        // each pointer is 2 bytes
        // to find the 3rd pointer in an interior, you start from position 12,
        // add 3 * 2 = 6, so your pointer position will be 18
        // you then read 18 and 19 to get the u16 of that pointer
        //
        let btree_header_size = if self.is_leaf() { 8 } else { 12 };
        // each pointer is u16 (2 bytes) but is cast to usize because rust expects
        // array indexes to be usize

        let pointer_index = self.offset + btree_header_size + (i * 2) as usize;

        u16::from_be_bytes([self.data[pointer_index], self.data[pointer_index + 1]]) as usize
    }
}
