use std::fs::File;

use crate::{btree, cell::Row};

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub rootpage: i64,
    pub column_names: Vec<String>,
}

pub fn parse_tables(file: &mut File, page_size: u16) -> Vec<Table> {
    let mut sqlite_master_rows: Vec<Row> = vec![];

    // read sqlite_master table
    btree::traverse(file, 1, page_size, &mut sqlite_master_rows);

    let mut tables: Vec<Table> = vec![];
    // save the table name and references
    for row in &sqlite_master_rows {
        // The table schema lives in the 5th column in sqlite_master
        if let Some(table_schema) = row.values[4].as_text() {
            let column_names = parse_column_names(table_schema);

            if row.values[0].as_text().unwrap() == "table" {
                tables.push(Table {
                    name: String::from(row.values[1].as_text().unwrap()),
                    rootpage: row.values[3].as_integer().unwrap(),
                    column_names,
                })
            }
        }
    }

    tables
}

fn parse_column_names(table_definition: &str) -> Vec<String> {
    // Find content between first ( and last )
    let start = table_definition.find('(').unwrap() + 1;
    let end = table_definition.rfind(')').unwrap();
    let inner = &table_definition[start..end];

    inner
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        // Skip constraints (FOREIGN KEY, PRIMARY KEY, etc.)
        .filter(|s| {
            !s.starts_with("FOREIGN")
                && !s.starts_with("PRIMARY")
                && !s.starts_with("UNIQUE")
                && !s.starts_with("CHECK")
        })
        // Get just the column name (first word)
        .filter_map(|s| s.split_whitespace().next())
        .map(|s| s.to_string())
        .collect()
}

#[test]
fn test_parse_column_names() {
    let data = "
        CREATE TABLE Customer_Ownership(
            customer_id INTEGER NOT NULL,
            vin INTEGER NOT NULL,
            purchase_date DATE NOT NULL,
            purchase_price INTEGER NOT NULL,
            warantee_expire_date DATE,
            dealer_id INTEGER NOT NULL,
            FOREIGN KEY (customer_id) REFERENCES Customers(customer_id),
            FOREIGN KEY (vin) REFERENCES Car_Vins(vin),
            FOREIGN KEY (dealer_id) REFERENCES Dealers(dealer_id)
            PRIMARY KEY (customer_id, vin)
        )
    ";

    let result = parse_column_names(data);

    assert_eq!("customer_id", *result.first().unwrap());
}
