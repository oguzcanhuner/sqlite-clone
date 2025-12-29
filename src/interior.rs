const HEADER_SIZE: usize = 100;
const BTREE_HEADER_SIZE: usize = 12;

pub struct Cell {
    pub child_page_number: u32,
    pub rowid: u8,
}

pub fn get_cell(i: u16, page: &[u8]) -> Cell {
    // each pointer is u16 (2 bytes) but is cast to usize because rust expects
    // array indexes to be usize
    let pointer_index = BTREE_HEADER_SIZE + (i * 2) as usize;
    let cell_offset =
        u16::from_be_bytes([page[pointer_index], page[pointer_index + 1]]) as usize - HEADER_SIZE;

    // the child page number is a u32 (4 bytes)
    let child_page = u32::from_be_bytes([
        page[cell_offset],
        page[cell_offset + 1],
        page[cell_offset + 2],
        page[cell_offset + 3],
    ]);

    let rowid_byte = page[cell_offset + 4];

    Cell {
        child_page_number: child_page,
        rowid: rowid_byte,
    }
}
