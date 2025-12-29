# SQLite Clone in Rust

A minimal SQLite implementation for learning purposes.

## Roadmap

### Reading (Query-Only)

- [x] Parse database header
- [x] Parse b-tree page headers
- [x] Parse interior page cells
- [ ] Parse leaf page cells
- [ ] Decode varints
- [ ] Decode record format (column types + values)
- [ ] Read `sqlite_master` to find tables
- [ ] Traverse table b-trees to read rows
- [ ] Parse basic SQL (`SELECT * FROM table`)
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
