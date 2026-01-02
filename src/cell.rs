use crate::value::{Value, parse_type_code};
use crate::varint::parse_varint;

pub struct Cell {
    pub child_page_number: u32,
    pub rowid: u64,
}

pub fn parse_interior_cell(index: usize, page: &[u8]) -> Cell {
    // the child page number is a u32 (4 bytes)
    let child_page = u32::from_be_bytes([
        page[index],
        page[index + 1],
        page[index + 2],
        page[index + 3],
    ]);

    // rowid is a varint (which means we don't know how many bytes the value takes up)
    // it can be up to 9 bytes. the parse_varint function takes all bytes from the current
    // offset (i.e. after we've read the 4 bytes which contains the child page number) up to
    // the end of the buffer (hence [cell_offset + 4..]).
    let (rowid, _bytes_read) = parse_varint(&page[index + 4..]);

    Cell {
        child_page_number: child_page,
        rowid,
    }
}

// the leaf page contains a header just like the first iterior page
// the structure is the same:
//
// | Offset | Size | Description |
// |--------|------|-------------|
// | 0 | 1 | Page type (0x0D = leaf table, 0x05 = interior table, 0x0A = leaf index, 0x02 = interior index) |
// | 1 | 2 | First freeblock offset (0 if none) |
// | 3 | 2 | Number of cells on this page |
// | 5 | 2 | Start of cell content area |
// | 7 | 1 | Fragmented free bytes |
// cell pointer data starts at offset 8
//
// the structure of a leaf cell:
// [varint: payload size] [varint: rowid] [payload]
//
// the payload:
// [varint: header size] [type codes...] [values...]
//
// - Header size — Total bytes in the header (including this varint)
// - Type codes — One varint per column, tells you the type and size
// - Values — The actual column data, packed together
//
// Example
// A row (id=5, name="Alice"):
// Payload:
//   Header size: 4
//   Type code: 1 (8-bit int)
//   Type code: 23 (text, length = (23-13)/2 = 5)
//   Value: 0x05 (the integer 5)
//   Value: "Alice" (5 bytes)
//
// Each column gets:
// 1. A type code in the header
// 2. A value in the values section
// Header:
//   Type code 1     → column 1 is an 8-bit integer
//   Type code 23    → column 2 is 5-byte text
// Values:
//   0x05            → column 1 value: 5
//   "Alice"         → column 2 value: Alice

#[derive(PartialEq, Debug)]
pub struct Row {
    pub rowid: u64,
    pub values: Vec<Value>,
}

// The index comes from the cell pointer array (which starts at offset 8)
// The header also contains number_of_cells
// Each index is a 2-byte pointer, so you need to fetch the two bytes and cast them
// together using big-endian.
pub fn parse_leaf_cell(pointer: usize, page: &[u8]) -> Row {
    // lets say the pointer is 300
    // Cell structure: [payload_size][rowid][payload]
    let (_payload_size, payload_bytes_read) = parse_varint(&page[pointer..]);
    let (rowid, rowid_bytes_read) = parse_varint(&page[(pointer + payload_bytes_read)..]);

    let payload_start = pointer + payload_bytes_read + rowid_bytes_read;

    // Payload structure: [header_size][type_codes...][values...]
    let (header_size, header_bytes_read) = parse_varint(&page[payload_start..]);

    // a type code goes up to 64 bytes
    let mut type_codes: Vec<u64> = vec![];

    // now that we have header_size, we first subtract bytes_read from it to get the
    // remaining bytes. we then keep reading type_codes (which are varints) until we
    // have read the remaining bytes. the cell header basically just contains type codes.
    // - Type codes — One varint per column, tells you the type and size
    let mut offset = header_bytes_read;

    while offset < header_size as usize {
        let (type_code, n) = parse_varint(&page[payload_start + offset..]);
        type_codes.push(type_code);
        offset += n;
    }

    let mut values: Vec<Value> = vec![];

    // now we have the type codes, we can start reading values
    // values start right after the header (which is at payload_start + header_size bytes)
    let mut values_offset = payload_start + header_size as usize;

    for type_code in type_codes {
        let (value, size) = parse_type_code(type_code, &page[values_offset..]);
        values.push(value);
        values_offset += size;
    }

    Row { rowid, values }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_interior_cell() {
        // Build a fake page. Fill it with 0s (0u8)
        let mut fake_page = vec![0u8; 1024];
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

        // parse_cell expects the actual cell offset (800), not the pointer array index
        let result = parse_interior_cell(800, &fake_page);

        assert_eq!(result.child_page_number, target_cell.child_page_number);
        assert_eq!(result.rowid, target_cell.rowid);
    }

    // Cell structure: [payload_size][rowid][payload]
    // Payload structure: [header_size][type_codes...][values...]

    #[test]
    fn test_parse_leaf_cell_i8() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..305].copy_from_slice(&[
            0x03, // payload_size = 3
            0x01, // rowid = 1
            0x02, // header_size = 2
            0x01, // type_code = 1 (i8)
            0x02, // value = 2
        ]);

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Integer(2)]);
    }

    #[test]
    fn test_parse_leaf_cell_i16() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..306].copy_from_slice(&[
            0x04, // payload_size = 4
            0x01, // rowid = 1
            0x02, // header_size = 2
            0x02, // type_code = 2 (i16)
            0x02, 0x02, // value = 514
        ]);

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Integer(514)]);
    }

    #[test]
    fn test_parse_leaf_cell_i24() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..307].copy_from_slice(&[
            0x05, // payload_size = 5
            0x01, // rowid = 1
            0x02, // header_size = 2
            0x03, // type_code = 3 (i24)
            0x00, 0x02, 0x02, // value = 514
        ]);

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Integer(514)]);
    }

    #[test]
    fn test_parse_leaf_cell_i32() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..308].copy_from_slice(&[
            0x06, // payload_size = 6
            0x01, // _rowid = 1
            0x02, // header_size = 2
            0x04, // type_code = 4 (i32)
            0x00, 0x00, 0x02, 0x02, // value = 514
        ]);

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Integer(514)]);
    }

    #[test]
    fn test_parse_leaf_cell_i48() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..310].copy_from_slice(&[
            0x08, // payload_size = 8
            0x01, // _rowid = 1
            0x02, // header_size = 2
            0x05, // type_code = 5 (i48)
            0x00, 0x00, 0x00, 0x00, 0x02, 0x02, // value = 514
        ]);

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Integer(514)]);
    }

    #[test]
    fn test_parse_leaf_cell_i64() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..312].copy_from_slice(&[
            0x0A, // payload_size = 10
            0x01, // _rowid = 1
            0x02, // header_size = 2
            0x06, // type_code = 6 (i64)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x02, // value = 514
        ]);

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Integer(514)]);
    }

    #[test]
    fn test_parse_leaf_cell_blob() {
        let mut fake_page = [0u8; 1024];
        // payload = header(3) + blob(144) = 147 = 0x93
        fake_page[300..306].copy_from_slice(&[
            0x81, 0x13, // payload_size = 147 (varint: 2 bytes)
            0x01, // _rowid = 1
            0x03, // header_size = 3
            0x82, 0x2C, // varint type_code = 300. BLOB type (>=12 even)
        ]);
        // size of the value is (300-12)/2 = 144
        fake_page[306..450].fill(b'C');

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Blob(vec![b'C'; 144])]);
    }

    #[test]
    fn test_parse_leaf_cell_multiple_values() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..308].copy_from_slice(&[
            0x06, // payload_size = 6
            0x01, // _rowid = 1
            0x03, // header_size = 3
            0x01, // type_code = 1 (i8)
            0x02, // type_code = 2 (i16)
            0x02, // value = 2
            0x02, 0x02, // value = 514
        ]);

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Integer(2), Value::Integer(514)]);
    }

    #[test]
    fn test_parse_leaf_cell_null() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..304].copy_from_slice(&[
            0x02, // payload_size = 2
            0x01, // _rowid = 1
            0x02, // header_size = 2
            0x00, // type_code = 0 (NULL)
        ]);

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Null]);
    }

    #[test]
    fn test_parse_leaf_cell_float() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..304].copy_from_slice(&[
            0x0A, // payload_size = 10
            0x01, // _rowid = 1
            0x02, // header_size = 2
            0x07, // type_code = 7 (float)
        ]);
        fake_page[304..312].copy_from_slice(&3.12_f64.to_be_bytes());

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Float(3.12)]);
    }

    #[test]
    fn test_parse_leaf_cell_literal_0() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..304].copy_from_slice(&[
            0x02, // payload_size = 2
            0x01, // _rowid = 1
            0x02, // header_size = 2
            0x08, // type_code = 8 (literal 0)
        ]);

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Integer(0)]);
    }

    #[test]
    fn test_parse_leaf_cell_literal_1() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..304].copy_from_slice(&[
            0x02, // payload_size = 2
            0x01, // _rowid = 1
            0x02, // header_size = 2
            0x09, // type_code = 9 (literal 1)
        ]);

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Integer(1)]);
    }

    #[test]
    fn test_parse_leaf_cell_text() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..304].copy_from_slice(&[
            0x07, // payload_size = 7
            0x01, // _rowid = 1
            0x02, // header_size = 2
            0x17, // type_code = 23 (text, len = (23-13)/2 = 5)
        ]);
        fake_page[304..309].copy_from_slice(b"Alice");

        let result = parse_leaf_cell(300, &fake_page);
        assert_eq!(result.rowid, 1);
        assert_eq!(result.values, vec![Value::Text("Alice".to_string())]);
    }
}
