# TODO: Preserve original column order

## Problem

The `process` pipeline in `src/parser.rs` currently emits the output table's
columns in **sorted** order, not the original header order. For an input like:

```
| name   | role      |
|--------|-----------|
| kurisu | bassist   |
| betty  | guitarist |
: job = (t, r, i) => `r["name"] the r["role"]`
```

the desired output keeps `name`, `role`, then the new `job` column at the end:

```
| name | role | job |
|---|---|---|
| kurisu | bassist | kurisu the bassist |
| betty | guitarist | betty the guitarist |
```

**Scope:** column ordering only. No column padding/alignment (the existing `dump`
joins cells with single spaces; keep that). Existing tests that assert sorted
output will be updated to assert order preservation.

## Root cause

1. `parse` and `extend` store each row as `std::collections::HashMap<String,
   String>`. A `HashMap` has **non-deterministic iteration order**, so the
   original column order is lost at parse time.
2. To compensate, `column_names` (parser.rs:187-198) collects keys across rows
   and calls `names.sort()` (line 196). This is what produces the visible
   "sorted columns" symptom in `dump` (parser.rs:209-233).

The fix is to carry column order through the whole pipeline with an ordered map,
then stop sorting.

`indexmap` 2.14.2 is already available as a transitive dependency (pulled in via
`boa_engine`, see `Cargo.lock:624`), so adding it as a direct dependency is
cheap and does not add a new crate to the build.

## Data-flow change

Replace `HashMap<String, String>` with `indexmap::IndexMap<String, String>`
everywhere a row/columns map is represented:

- `indexmap::IndexMap` preserves insertion order.
- It implements `Deref<Target = HashMap<String, String>>` and is otherwise a
  drop-in: `.get`, `.insert`, `.clone`, `.iter()`, equality, and iteration over
  `(key, value)` all work unchanged.
- It requires the `std` feature (default in indexmap 2.x), which is on.

Insertion order to preserve:
- `parse`: insert columns in header order (already enumerated by index,
  parser.rs:50-53) → order is preserved automatically once the type is ordered.
- `extend`: start each row from a clone of the original row, then `insert`
  each extension column in the order of the `extensions` slice (parser.rs:125-139)
  → original columns keep their order and new columns append at the end. This
  already matches the desired behavior; the only thing broken was `dump`.

## Implementation steps

### 1. Add the dependency

- `Cargo.toml`: add `indexmap = "2"` to `[dependencies]`.
- Run `cargo build` to refresh `Cargo.lock` (indexmap 2.14.2 already present).

### 2. Switch the row type to `IndexMap`

In `src/parser.rs`, replace `use std::collections::HashMap;` with
`use indexmap::IndexMap;` and update the signatures:

- `parse` → `Result<Vec<IndexMap<String, String>>, String>`
  - `let mut rows: Vec<IndexMap<String, String>> = Vec::new();`
  - `let mut row = IndexMap::new();`
- `extend` →
  `inlet: &[IndexMap<String, String>]`,
  `result: Vec<IndexMap<String, String>>`
- `dump` → `pub fn dump(rows: &[IndexMap<String, String>]) -> String`
- `column_names` → `fn column_names(rows: &[IndexMap<String, String>]) -> Vec<String>`

In `src/js.rs`, replace `use std::collections::HashMap;` with
`use indexmap::IndexMap;` and update the signatures:

- `JsRule::apply` → `table: &[IndexMap<String, String>]`,
  `row: &IndexMap<String, String>`
- `build_table` → `table: &[IndexMap<String, String>]`
- `build_row` → `row: &IndexMap<String, String>`

Check for any other `HashMap` references (the `mod tests` blocks in both files
construct `HashMap::new()` and will need to switch to `IndexMap::new()`).

### 3. Remove the sort in `column_names`

In `column_names` (parser.rs:187-198), delete the `names.sort();` call and update
the doc comment (lines 184-186) to say columns follow original order. The
dedup logic (`names.iter().all(...)`) is fine, but since `extend` builds one
`IndexMap` per row in the same order and `dump` iterates the union, prefer
deriving the order from the **first row's** key order to be unambiguous:

- Iterate rows in order; for each row iterate its keys (insertion order) and
  append each key the first time it is seen. Drop the final sort.

### 4. Update `dump` doc comment

The doc comment for `dump` (parser.rs:200-208) says columns are "sorted to
produce deterministic output". Change it to say columns follow original order.

### 5. Update existing tests

In `src/parser.rs` tests, change `HashMap` to `IndexMap` and fix the expected
strings:

- `test_dump_basic_table` (parser.rs:332-351): insert order is `name`, `age`,
  `job`; expected becomes
  `| name | age | job |\n|---|---|---|\n| Mira | 27 | Superhero |\n| Pietro | 24 | Clown |\n| Flora | 18 | Singer |\n`.
- `test_dump_missing_cell_renders_empty` (parser.rs:376-386): `row` inserts
  `name`, `age`; `other` inserts `job`. Union order (first-seen) is `name`,
  `age`, `job`; expected becomes
  `| name | age | job |\n|---|---|---|\n| Mira | 27 |  |\n|  |  | Singer |\n`.
- `test_dump_columns_are_sorted` (parser.rs:388-396): rename to
  `test_dump_columns_preserve_order`, insert `zebra` then `alpha`, expect
  `| zebra | alpha |\n|---|---|\n| 1 | 2 |\n`.
- `test_dump_empty_rows` (parser.rs:369-373): type the empty vec as
  `Vec<IndexMap<String, String>>`.
- `test_extend_no_extensions_is_identity` and the `extend` tests: switch
  `HashMap::new()` → `IndexMap::new()`.
- `test_dump_matches_parse_roundtrip`: unchanged logic, just compiles against
  the new type.

In `src/js.rs` tests, switch `HashMap::new()` → `IndexMap::new()` in
`sample_table` (js.rs:112-126) and any other construction.

### 6. Add a regression test for the task's example

Add a test in `parser.rs` that runs the full `process` on the task's example
table and asserts the output preserves order `name`, `role`, `job`:

```
| name | role | job |
|---|---|---|
| kurisu | bassist | kurisu the bassist |
| betty | guitarist | betty the guitarist |
```

## Verification

- `cargo test` — all tests pass, including the new order-preservation test.
- `cargo build` — clean compile.
- Manual smoke test with the task's example piped through `mdt.exe` (or
  `cargo run`) and confirm columns appear as `name | role | job`.

## Notes / risks

- `IndexMap` is a drop-in for the `HashMap` usages here, so the change is
  localized to the two source files plus `Cargo.toml`.
- The JS-facing code (`build_row`) iterates the map to set object properties;
  JS objects don't depend on insertion order for property access, so this is safe.
- No padding is added per the agreed scope; only ordering changes.
