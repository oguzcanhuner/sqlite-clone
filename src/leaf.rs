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
// The type code tells you what kind of data and how big:
// | Code | Meaning |
// |------|---------|
// | 0 | NULL (0 bytes) |
// | 1 | 8-bit integer (1 byte) |
// | 2 | 16-bit integer (2 bytes) |
// | 3 | 24-bit integer (3 bytes) |
// | 4 | 32-bit integer (4 bytes) |
// | 5 | 48-bit integer (6 bytes) |
// | 6 | 64-bit integer (8 bytes) |
// | 7 | float (8 bytes) |
// | 8 | literal 0 (0 bytes) |
// | 9 | literal 1 (0 bytes) |
// | ≥12, even | BLOB, size = (code-12)/2 |
// | ≥13, odd | TEXT, size = (code-13)/2 |
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
//
//

use crate::database::parse_varint;

#[derive(PartialEq, Debug)]
enum Value {
    Null,
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
}

struct Cell {
    values: Vec<Value>,
}

// The index comes from the cell pointer array (which starts at offset 8)
// The header also contains number_of_cells
// Each index is a 2-byte pointer, so you need to fetch the two bytes and cast them
// together using big-endian.
fn parse_cell(pointer: usize, page: &[u8]) -> Cell {
    // lets say the pointer is 300
    // we first have a varint header size
    let (header_size, bytes_read) = parse_varint(&page[pointer..]);

    println!("Header size: {}. Bytes read: {}", header_size, bytes_read);

    // a type code goes up to 64 bytes
    let mut type_codes: Vec<u64> = vec![];
    let mut values: Vec<Value> = vec![];

    // now that we have header_size, we first subtract bytes_read from it to get the
    // remaining bytes. we then keep reading type_codes (which are varints) until we
    // have read the remaining bytes. the cell header basically just contains type codes.
    // - Type codes — One varint per column, tells you the type and size
    let mut offset = bytes_read;

    println!("Offset: {}", offset);
    while offset < header_size as usize {
        let (type_code, n) = parse_varint(&page[pointer + offset..]);
        type_codes.push(type_code);
        offset += n;
        println!("Offset: {}", offset);
    }

    println!("Type codes: {:?}", type_codes);

    // now we have the type codes, we can start reading values
    let mut values_offset = pointer + offset;

    for type_code in type_codes {
        let (value, size) = parse_type_code(type_code, &page[values_offset..]);
        values.push(value);
        values_offset += size;
    }

    Cell { values }

    // we receive an array of bytes which make up a page
    // we also receive an index
    // first go to the index i
    //return something
}

// | 0 | NULL (0 bytes) |
// | 1 | 8-bit integer (1 byte) |
// | 2 | 16-bit integer (2 bytes) |
// | 3 | 24-bit integer (3 bytes) |
// | 4 | 32-bit integer (4 bytes) |
// | 5 | 48-bit integer (6 bytes) |
// | 6 | 64-bit integer (8 bytes) |
// | 7 | float (8 bytes) |
// | 8 | literal 0 (0 bytes) |
// | 9 | literal 1 (0 bytes) |
// | ≥12, even | BLOB, size = (code-12)/2 |
// | ≥13, odd | TEXT, size = (code-13)/2 |
fn parse_type_code(type_code: u64, page: &[u8]) -> (Value, usize) {
    match type_code {
        0 => (Value::Null, 0),
        1 => (Value::Integer(page[0] as i8 as i64), 1),
        2 => (
            Value::Integer(i16::from_be_bytes([page[0], page[1]]) as i64),
            2,
        ),
        3 => (
            Value::Integer(i32::from_be_bytes([0, page[0], page[1], page[2]]) as i64),
            3,
        ),
        4 => (
            Value::Integer(i32::from_be_bytes([page[0], page[1], page[2], page[3]]) as i64),
            4,
        ),
        5 => (
            // TODO: need to handle sign extension. this code currently only supports positive
            // integers
            Value::Integer(i64::from_be_bytes([
                0, 0, page[0], page[1], page[2], page[3], page[4], page[5],
            ])),
            6,
        ),
        6 => (
            Value::Integer(i64::from_be_bytes([
                page[0], page[1], page[2], page[3], page[4], page[5], page[6], page[7],
            ])),
            8,
        ),
        7 => (
            Value::Float(f64::from_be_bytes([
                page[0], page[1], page[2], page[3], page[4], page[5], page[6], page[7],
            ])),
            8,
        ),
        8 => (Value::Integer(0), 0),
        9 => (Value::Integer(1), 0),
        n if n >= 12 && n % 2 == 0 => {
            let len = ((n - 12) / 2) as usize;
            (Value::Blob(page[..len].to_vec()), len)
        }
        n if n >= 13 && n % 2 == 1 => {
            let len = ((n - 13) / 2) as usize;
            (
                Value::Text(String::from_utf8_lossy(&page[..len]).to_string()),
                len,
            )
        }
        _ => panic!("Unknown type code: {}", type_code),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_cell_i8() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..303].copy_from_slice(&[
            0x02, // header_size = 2
            0x01, // type_code = 1 (i8)
            0x02, // value = 2
        ]);

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Integer(2)]);
    }

    #[test]
    fn test_parse_cell_i16() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..304].copy_from_slice(&[
            0x02, // header_size = 2
            0x02, // type_code = 2 (i16)
            0x02, 0x02, // value = 514
        ]);

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Integer(514)]);
    }

    #[test]
    fn test_parse_cell_i24() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..305].copy_from_slice(&[
            0x02, // header_size = 2
            0x03, // type_code = 3 (i24)
            0x00, 0x02, 0x02, // value = 514
        ]);

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Integer(514)]);
    }

    #[test]
    fn test_parse_cell_i32() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..306].copy_from_slice(&[
            0x02, // header_size = 2
            0x04, // type_code = 4 (i32)
            0x00, 0x00, 0x02, 0x02, // value = 514
        ]);

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Integer(514)]);
    }

    #[test]
    fn test_parse_cell_i48() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..308].copy_from_slice(&[
            0x02, // header_size = 2
            0x05, // type_code = 5 (i48)
            0x00, 0x00, 0x00, 0x00, 0x02, 0x02, // value = 514
        ]);

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Integer(514)]);
    }

    #[test]
    fn test_parse_cell_i64() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..310].copy_from_slice(&[
            0x02, // header_size = 2
            0x06, // type_code = 6 (i64)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x02, // value = 514
        ]);

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Integer(514)]);
    }

    #[test]
    fn test_parse_cell_blob() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..303].copy_from_slice(&[
            0x03, // header_size = 3
            0x82, 0x2C, // varint type_code = 300. BLOB type (>=12 even)
        ]);
        // size of the value is (300-12)/2 = 144
        fake_page[303..447].fill(b'C');

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Blob(vec![b'C'; 144])]);
    }

    #[test]
    fn test_parse_cell_multiple_values() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..306].copy_from_slice(&[
            0x03, // header_size = 3
            0x01, // type_code = 1 (i8)
            0x02, // type_code = 2 (i16)
            0x02, // value = 2
            0x02, 0x02, // value = 514
        ]);

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Integer(2), Value::Integer(514)]);
    }

    #[test]
    fn test_parse_cell_null() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..302].copy_from_slice(&[
            0x02, // header_size = 2
            0x00, // type_code = 0 (NULL)
        ]);

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Null]);
    }

    #[test]
    fn test_parse_cell_float() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..302].copy_from_slice(&[
            0x02, // header_size = 2
            0x07, // type_code = 7 (float)
        ]);
        fake_page[302..310].copy_from_slice(&3.12_f64.to_be_bytes());

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Float(3.12)]);
    }

    #[test]
    fn test_parse_cell_literal_0() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..302].copy_from_slice(&[
            0x02, // header_size = 2
            0x08, // type_code = 8 (literal 0)
        ]);

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Integer(0)]);
    }

    #[test]
    fn test_parse_cell_literal_1() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..302].copy_from_slice(&[
            0x02, // header_size = 2
            0x09, // type_code = 9 (literal 1)
        ]);

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Integer(1)]);
    }

    #[test]
    fn test_parse_cell_text() {
        let mut fake_page = [0u8; 1024];
        fake_page[300..302].copy_from_slice(&[
            0x02, // header_size = 2
            0x17, // type_code = 23 (text, len = (23-13)/2 = 5)
        ]);
        fake_page[302..307].copy_from_slice(b"Alice");

        let result = parse_cell(300, &fake_page);
        assert_eq!(result.values, vec![Value::Text("Alice".to_string())]);
    }
}
