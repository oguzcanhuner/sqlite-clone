use std::fs::File;

use crate::schema::Table;

pub struct Db {
    pub file: File,
    pub page_size: u16,
    pub tables: Vec<Table>,
}
