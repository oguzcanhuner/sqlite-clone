use std::fs::File;

use crate::{
    cell::{self, Row},
    page::Page,
};

// follow the cell references in interior pages and fetch values from
// linked leaf pages
pub fn traverse(file: &mut File, page_num: u32, page_size: u16, rows: &mut Vec<Row>) {
    let page = Page::read(file, page_num, page_size);

    if page.is_leaf() {
        for i in 0..page.num_cells {
            let row = cell::parse_leaf_cell(page.cell_pointer(i), &page.data);

            rows.push(row);
        }
    } else {
        for i in 0..page.num_cells {
            let cell = cell::parse_interior_cell(page.cell_pointer(i), &page.data);

            traverse(file, cell.child_page_number, page_size, rows);
        }

        traverse(file, page.rightmost_child(), page_size, rows);
    }
}
