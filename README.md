# SQLite Clone in Rust

A minimal SQLite implementation for learning purposes.

## Roadmap

### Reading (Query-Only)

- [x] Parse database header
- [x] Parse b-tree page headers
- [x] Parse interior page cells
- [x] Parse leaf page cells
- [x] Decode varints
- [x] Decode record format (column types + values)
- [x] Read `sqlite_master` to find tables
- [x] Traverse table b-trees to read rows
- [x] Parse basic SQL (`SELECT * FROM table`)
- [ ] Parse table column names
- [ ] Filter rows (`WHERE` clause)
- [ ] Parse indexes
- [ ] Use indexes for faster lookups

### Writing

- [ ] Create new database file
- [ ] Write header
- [ ] Insert rows (append to leaf pages)
- [ ] Page splitting (when a page fills up)
- [ ] Update rows
- [ ] Delete rows
- [ ] Manage free pages (freelist)
- [ ] Create tables (`CREATE TABLE`)
- [ ] Create indexes (`CREATE INDEX`)

### Advanced

- [ ] Transactions / rollback journal
- [ ] Multiple column indexes
- [ ] `JOIN` queries
- [ ] Aggregations (`COUNT`, `SUM`, etc.)
- [ ] `ORDER BY`
- [ ] Expression evaluation
