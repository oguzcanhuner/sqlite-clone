use std::{fs::File, io::Read};

// An example of how value 300 is decoded, encoded as [0x82, 0x2C]
//
// hex characters are 4 bits each.
//
// Byte 1: 0x82 (8 == 1000 2 == 0010)
// 0x82 = 10000010
// High bit is 1 → continue
// Data bits: 0000010 → 2
//
// Byte 2: 0x2C (2 == 0010 C == 1100)
// 0x2C = 00101100
// High bit is 0 → stop
// Data bits: 0101100 → 44
//
// Combine:
//
// First chunk:  0000010 (2)
// Second chunk: 0101100 (44)
// Concatenate: 00000100101100
//              = 0000010 0101100
//              = 256 + 32 + 8 + 4
//              = 300
//
// bytes would typically be an array of bytes from the current position until
// the end of a buffer. This is because you don't know the length of the int yet.
// we rely on the high bit to tell us when to stop parsing the int value.
// we then return a `bytes_read` value which tells us how many bytes we need to shift
// to start reading the next value.
pub fn parse_varint(bytes: &[u8]) -> (u64, usize) {
    let mut value: u64 = 0;
    let mut bytes_read = 0;

    for &byte in bytes.iter().take(9) {
        // max 9 bytes
        bytes_read += 1;

        // 0x80 == 8 = 1000 + 0 = 0000 == 10000000
        // doing a bitwise AND (&) operation compares each value
        // at the same posision. For example:
        // 01110000 & 1000000 == 00000000
        // 10101010 & 1000000 == 10000000
        // (so 1 needs to be in both positions for the result to be 1)
        // to check if the high bit (bit 7) is 1 or 0, we can simply
        // do a bitwise AND with 0x80 (10000000)
        let high_bit_set = byte & 0x80 != 0;

        // we use a similar trick to get all of the "data bits" - these are bits 0-6 (excluding
        // the 7th (high) bit).
        // 0x7F == 7 = 0111 + F(15) = 1111 == 01111111
        // this is like the opposite of 0x80 which isolates only one bit - this isolates all the
        // other bits. examples:
        //
        // 01110000 & 01111111 == 01110000
        // 10101010 & 01111111 == 00101010 (the first 1 bit is set to 0)
        let data_bits = byte & 0x7F;

        // shifting left by 7 moves all bits 7 positions to the left:
        // let's say that in the first iteration, the value is 2
        // in the second iteration, value << 7 will do this:
        // before: 00000000 00000010  (2)
        // after:  00000001 00000000  (256)
        // this "makes room" on the right for the next 7 bits.
        let shifted_value = value << 7;

        // lets say that the data_bits for this iteration is 44. value = 256 | 44
        // OR combines them — where either has a 1, the result has a 1:
        // 00000001 00000000  (256)
        // | 00000000 00101100  (44)
        // -------------------
        // 00000001 00101100  (300)
        value = shifted_value | data_bits as u64;

        if !high_bit_set {
            break;
        }
    }

    // at this point, we have mutated both value and bytes_read
    // value is a u64 which contains the actual value of the varint
    // bytes_read is a usize (since it'll be used as an index) which tells the caller
    // how many bytes this varint is made up of (so that the caller can move to
    // the correct offset for the next value)
    (value, bytes_read)
}

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

    // super refers to the parent module (this file). * means everything.
    // so this line makes every function / struct etc defined in this file available within
    // the tests module.
    use super::*;

    // #[test] is an attribute that the compiler understands. attributes are typically
    // used to mark something for the compiler or other tools.
    #[test]
    fn test_parse_varint() {
        let result = parse_varint(&[0x82, 0x2C]);
        assert_eq!(result, (300, 2));
    }

    #[test]
    fn test_parse_varint_zero() {
        let result = parse_varint(&[0x00]);
        assert_eq!(result, (0, 1));
    }

    #[test]
    fn test_parse_varint_large() {
        // 16384 = 0x4000, encoded as [0x81, 0x80, 0x00]
        let result = parse_varint(&[0x81, 0x80, 0x00]);
        assert_eq!(result, (16384, 3));
    }

    #[test]
    // test that parse_header returns a Header with a page size
    fn test_parse_header() {
        let mut file = File::open("chinook.db").unwrap();

        let result = parse_header(&mut file);

        assert_eq!(result.page_size, 1024)
    }
}
