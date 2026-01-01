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

#[derive(PartialEq, Debug)]
pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl Value {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }
}

pub fn parse_type_code(type_code: u64, data: &[u8]) -> (Value, usize) {
    match type_code {
        0 => (Value::Null, 0),
        1 => (Value::Integer(data[0] as i8 as i64), 1),
        2 => (
            Value::Integer(i16::from_be_bytes([data[0], data[1]]) as i64),
            2,
        ),
        3 => (
            Value::Integer(i32::from_be_bytes([0, data[0], data[1], data[2]]) as i64),
            3,
        ),
        4 => (
            Value::Integer(i32::from_be_bytes([data[0], data[1], data[2], data[3]]) as i64),
            4,
        ),
        5 => (
            // TODO: need to handle sign extension. this code currently only supports positive
            // integers
            Value::Integer(i64::from_be_bytes([
                0, 0, data[0], data[1], data[2], data[3], data[4], data[5],
            ])),
            6,
        ),
        6 => (
            Value::Integer(i64::from_be_bytes([
                data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
            ])),
            8,
        ),
        7 => (
            Value::Float(f64::from_be_bytes([
                data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
            ])),
            8,
        ),
        8 => (Value::Integer(0), 0),
        9 => (Value::Integer(1), 0),
        n if n >= 12 && n % 2 == 0 => {
            let len = ((n - 12) / 2) as usize;
            (Value::Blob(data[..len].to_vec()), len)
        }
        n if n >= 13 && n % 2 == 1 => {
            let len = ((n - 13) / 2) as usize;
            (
                Value::Text(String::from_utf8_lossy(&data[..len]).to_string()),
                len,
            )
        }
        _ => panic!("Unknown type code: {}", type_code),
    }
}
