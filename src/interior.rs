use crate::database::parse_varint;

const HEADER_SIZE: usize = 100;
const BTREE_HEADER_SIZE: usize = 12;

pub struct Cell {
    pub child_page_number: u32,
    pub rowid: u64,
}

pub fn parse_cell(i: u16, page: &[u8]) -> Cell {
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

    // rowid is a varint (which means we don't know how many bytes the value takes up)
    // it can be up to 9 bytes. the parse_varint function takes all bytes from the current
    // offset (i.e. after we've read the 4 bytes which contains the child page number) up to
    // the end of the buffer (hence [cell_offset + 4..]).
    let (rowid, _bytes_read) = parse_varint(&page[cell_offset + 4..]);

    Cell {
        child_page_number: child_page,
        rowid,
    }
}

#[test]
fn test_parse_cell() {
    // Build a fake page (starting at offset 100, so no db header). Fill it with 0s (0u8)
    let mut fake_page = vec![0u8; 1024 - HEADER_SIZE];
    // cell pointer is at index 12-13. anything before is the header.
    // the pointer should be index 900 (which needs two bytes)
    //
    // the first byte is the most "significant", so we multiply it by 256
    // the second byte is then added to this
    // 0x03 == 3 and then times is by 256 = 768
    // 0x84 == 132
    // 768 + 132 == 900
    fake_page[12] = 0x03;
    fake_page[13] = 0x84;

    // first four bytes contain the page number (3)
    // note - since we offset by 100, the pointer 900 is referring to index 800 in our page buffer
    fake_page[800] = 0x00;
    fake_page[801] = 0x00;
    fake_page[802] = 0x00;
    fake_page[803] = 0x03;

    // then a varint contains the rowid. these two bytes, when encoded with the
    // varint algorithm, represent 300
    fake_page[804] = 0x82;
    fake_page[805] = 0x2C;

    let target_cell = Cell {
        child_page_number: 3,
        rowid: 300,
    };

    let result = parse_cell(0, &fake_page);

    assert_eq!(result.child_page_number, target_cell.child_page_number);
    assert_eq!(result.rowid, target_cell.rowid);
}
