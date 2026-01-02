use sqlite::run;
use sqlite::{Row, Value};

#[test]
fn test_select_all_albums() {
    let file_path = String::from("tests/chinook.db");
    let query = String::from("SELECT * FROM albums");

    let rows = run(&file_path, &query);

    println!("{:?}", rows.first().unwrap());

    let target_row: Row = Row {
        rowid: 1,
        values: vec![
            Value::Null,
            Value::Text(String::from("For Those About To Rock We Salute You")),
            Value::Integer(1),
        ],
    };

    assert_eq!(rows.first().unwrap(), &target_row);
    assert_eq!(rows.len(), 347 as usize);
}
