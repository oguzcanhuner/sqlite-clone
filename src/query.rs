use crate::{btree, cell::Row, db::Db};

pub fn execute(db: &mut Db, query: String) -> Vec<Row> {
    let mut rows: Vec<Row> = vec![];

    let parts: Vec<&str> = query.split_whitespace().collect();

    let mut from_index: usize = 0;
    let mut select_start_index: usize = 0;

    if let Some(position) = parts.iter().position(|string| *string == "SELECT") {
        select_start_index = position;
    }
    // find the word FROM and select the token after it
    if let Some(position) = parts.iter().position(|string| *string == "FROM") {
        from_index = position;
    }

    let table_name = parts[from_index + 1].to_string();
    let condition = parts[select_start_index + 1].to_string();

    if condition == "*"
        && let Some(table) = db.tables.iter().find(|t| t.name == table_name)
    {
        btree::traverse(&mut db.file, table.rootpage as u32, db.page_size, &mut rows);
    }

    rows
}
