use std::env::args;

use comfy_table::Table;
use sqlite::run;

fn main() {
    // This forms the basis of our CLI interface
    // Usage:
    // sqlite_oz query database_file_path
    //
    // query: A string which contains an SQL query
    // database_file_file: A path to a .db file containing a
    //   clustered index (i.e. values are within the b-tree)
    //
    // Example:
    // sqlite_oz "SELECT * FROM albums WHERE name = "something" chinook.db

    // collect all args into a list
    let args: Vec<String> = args().collect();

    // the first item is the query
    // the second item is the database name
    let query = args
        .get(1)
        .expect("Usage: sqlite_oz query database_file_path");
    let file_path = args
        .get(2)
        .expect("Usage: sqlite_oz query database_file_path");

    let rows = run(file_path, query);

    let mut table = Table::new();

    // TODO get the actual headers from schema. This should probably live in the run function.
    let headers = vec!["Unknown"; rows[0].values.len() + 1];

    table.set_header(&headers);
    for row in &rows {
        let mut values = vec![format!("{:?}", row.rowid)];
        values.extend(row.values.iter().map(|v| format!("{:?}", v)));

        table.add_row(values);
    }

    println!("{}", table);
}
