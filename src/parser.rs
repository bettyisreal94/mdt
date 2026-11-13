use std::collections::HashMap;

use boa_engine::Context;
use crate::js::{str_to_fn, JsRule};

/// Parse an extended markdown table string into a list of row maps.
///
/// Each returned map maps a column name (from the header row) to the cell
/// value in that column. Extension lines (starting with `:`) are ignored for
/// now. Returns an `Err` describing why parsing failed.
pub fn parse(table_str: &str) -> Result<Vec<HashMap<String, String>>, String> {
    let lines: Vec<&str> = table_str.lines().collect();

    // Collect the meaningful lines, skipping blank lines and extension lines.
    let mut meaningful: Vec<&str> = Vec::new();
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with(':') {
            continue;
        }
        meaningful.push(line);
    }

    // A valid table needs at least a header row and a separator row.
    if meaningful.len() < 2 {
        return Err("Table must have at least a header row and a separator".to_string());
    }

    let headers = parse_cells(meaningful[0])?;
    if headers.is_empty() {
        return Err("Header row must have at least one column".to_string());
    }

    let mut rows: Vec<HashMap<String, String>> = Vec::new();

    // The line immediately after the header must be the separator row.
    let mut iter = meaningful.iter().skip(1);
    if let Some(separator) = iter.next() {
        if !is_separator(separator) {
            return Err("Missing separator row after header".to_string());
        }
    }

    for line in iter {
        let cells = parse_cells(line)?;
        let mut row = HashMap::new();
        for (i, header) in headers.iter().enumerate() {
            let value = cells.get(i).map(|s| s.as_str()).unwrap_or("");
            row.insert(header.clone(), value.to_string());
        }
        rows.push(row);
    }

    Ok(rows)
}

fn parse_cells(line: &str) -> Result<Vec<String>, String> {
    let trimmed = line.trim();
    let clean = trimmed
        .trim_start_matches('|')
        .trim_end_matches('|')
        .trim();

    if clean.is_empty() {
        return Err("Empty row".to_string());
    }

    Ok(clean
        .split('|')
        .map(|s| s.trim().to_string())
        .collect())
}

/// Extract the extension definitions from a table string.
///
/// Each extension line starts with `:`, followed by the new column name,
/// an equals sign `=`, and finally the code for that column. Returns a
/// list of `(name, rule)` pairs, where `rule` is the parsed JavaScript
/// function produced from the extension code. Returns an `Err` if an
/// extension line is missing the equals sign, or if any extension code
/// cannot be compiled into a callable rule.
pub fn extract_extensions(table_str: &str) -> Result<Vec<(String, JsRule)>, String> {
    let mut extensions: Vec<(String, JsRule)> = Vec::new();

    for line in table_str.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(':') {
            continue;
        }

        let clean = trimmed[1..].trim();
        let (name, code) = clean
            .split_once('=')
            .map(|(name, code)| (name.trim().to_string(), code.trim().to_string()))
            .ok_or_else(|| "Extension line must contain '='".to_string())?;

        let rule = str_to_fn(&code)?;

        extensions.push((name, rule));
    }

    Ok(extensions)
}

fn is_separator(line: &str) -> bool {
    let trimmed = line.trim().trim_start_matches('|').trim_end_matches('|').trim();
    if trimmed.is_empty() {
        return false;
    }
    let mut cells = trimmed.split('|');
    for cell in cells.by_ref() {
        let c = cell.trim();
        if c.is_empty() {
            return false;
        }
        if !c.chars().all(|ch| ch == '-' || ch == ':') {
            return false;
        }
    }
    cells.next().is_none()
}

/// Return the sorted, de-duplicated set of column names present across all
/// rows. This gives a deterministic column order regardless of the (unordered)
/// `HashMap` iteration order.
fn column_names(rows: &[HashMap<String, String>]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for row in rows {
        for key in row.keys() {
            if names.iter().all(|n| n != key) {
                names.push(key.clone());
            }
        }
    }
    names.sort();
    names
}

/// Render a list of row maps as an extended markdown table string.
///
/// This is the inverse of [`parse`]: given the parsed rows (each mapping a
/// column name to its cell value) it regenerates the markdown table. Column
/// names are taken from the union of all row keys and sorted to produce
/// deterministic output.
///
/// The returned string is the header row, a `---` separator row and one row per
/// input map, each terminated by a newline.
pub fn dump(rows: &[HashMap<String, String>]) -> String {
    let names = column_names(rows);

    let mut out = String::new();

    // Header row: `| name | age | job |`
    if !names.is_empty() {
        let header_cells: Vec<&str> = names.iter().map(String::as_str).collect();
        out.push_str(&format!("| {} |\n", header_cells.join(" | ")));

        // Separator row: `|---|---|---|`
        let sep_cells: Vec<&str> = std::iter::repeat("---").take(names.len()).collect();
        out.push_str(&format!("|{}|\n", sep_cells.join("|")));
    }

    // Data rows
    for row in rows {
        let cells: Vec<String> = names.iter().map(|name| {
            row.get(name).map(String::as_str).unwrap_or("").to_string()
        }).collect();
        out.push_str(&format!("| {} |\n", cells.join(" | ")));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_table() {
        let table_str = r#"| name   | age | job       |
|--------|-----|-----------|
| Mira   | 27  | Superhero |
| Pietro | 24  | Clown     |
| Flora  | 18  | Singer    |"#;

        let rows = parse(table_str).expect("Failed to parse basic table");

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].get("name").unwrap(), "Mira");
        assert_eq!(rows[0].get("age").unwrap(), "27");
        assert_eq!(rows[0].get("job").unwrap(), "Superhero");
        assert_eq!(rows[1].get("name").unwrap(), "Pietro");
        assert_eq!(rows[2].get("job").unwrap(), "Singer");
    }

    #[test]
    fn test_parse_ignores_extension_lines() {
        let table_str = r#"| name   | age | job       |
|--------|-----|-----------|
| Mira   | 27  | Superhero |
| Pietro | 24  | Clown     |
| Flora  | 18  | Singer    |
: title = (t, r, i) => `${r["name"]}, the ${r["job"]}`"#;

        let rows = parse(table_str).expect("Failed to parse table with extension lines");

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].get("name").unwrap(), "Mira");
    }


    #[test]
    fn test_extract_multiple_extensions() {
        let table_str = r#"| name | age |
|------|-----|
| A    | 1   |
: first = r["name"]
: double = r["age"] * 2
: upper  = r["name"].upper()"#;

        let extensions = extract_extensions(table_str)
            .expect("Failed to extract extensions")
            .into_iter()
            .collect::<Vec<_>>();

        assert_eq!(extensions.len(), 3);
        assert_eq!(extensions[0].0, "first");
        assert_eq!(extensions[0].1.source(), "r[\"name\"]");
        assert_eq!(extensions[1].0, "double");
        assert_eq!(extensions[2].0, "upper");
        assert_eq!(extensions[2].1.source(), "r[\"name\"].upper()");

        // The extracted rules must be valid, callable JavaScript functions.
        for (_, rule) in &extensions {
            let rule = str_to_fn(rule.source()).expect("rule should be a valid function");
            let mut context = Context::default();
            let table: Vec<HashMap<String, String>> = vec![
                {
                    let mut row = HashMap::new();
                    row.insert("name".to_string(), "A".to_string());
                    row.insert("age".to_string(), "1".to_string());
                    row
                },
            ];
            rule.apply(&mut context, &table, &table[0], 0)
                .expect("rule should apply successfully");
        }
    }

    #[test]
    fn test_extract_extensions_no_equals_fails() {
        let table_str = ": broken line without equals";
        let result = extract_extensions(table_str);
        assert!(result.is_err(), "should fail without '='");
        match result {
            Err(err) => assert!(err.contains("='"), "got: {err}"),
            Ok(_) => panic!("expected an error"),
        }
    }

    #[test]
    fn test_extract_extensions_invalid_code_fails() {
        let table_str = ": good = r[\"name\"]\n: bad = this is not (valid javascript";
        let result = extract_extensions(table_str);
        assert!(result.is_err(), "should fail on invalid code");
        match result {
            Err(err) => assert!(err.contains("Failed to parse function"), "got: {err}"),
            Ok(_) => panic!("expected an error"),
        }
    }

    #[test]
    fn test_extract_empty_string() {
        assert!(extract_extensions("").unwrap().is_empty());
    }

    #[test]
    fn test_parse_missing_separator_fails() {
        let table_str = r#"| name | age |
| Mira | 27  |"#;

        let result = parse(table_str).expect_err("Should fail without a separator row");
        assert!(result.contains("Missing separator row"));
    }

    #[test]
    fn test_dump_basic_table() {
        let mut rows = HashMap::new();
        rows.insert("name".to_string(), "Mira".to_string());
        rows.insert("age".to_string(), "27".to_string());
        rows.insert("job".to_string(), "Superhero".to_string());

        let mut rows2 = HashMap::new();
        rows2.insert("name".to_string(), "Pietro".to_string());
        rows2.insert("age".to_string(), "24".to_string());
        rows2.insert("job".to_string(), "Clown".to_string());

        let mut rows3 = HashMap::new();
        rows3.insert("name".to_string(), "Flora".to_string());
        rows3.insert("age".to_string(), "18".to_string());
        rows3.insert("job".to_string(), "Singer".to_string());

        let expected = "| age | job | name |\n|---|---|---|\n| 27 | Superhero | Mira |\n| 24 | Clown | Pietro |\n| 18 | Singer | Flora |\n";

        assert_eq!(dump(&[rows, rows2, rows3]), expected);
    }

    #[test]
    fn test_dump_matches_parse_roundtrip() {
        let table_str = r#"| name   | age | job       |
|--------|-----|-----------|
| Mira   | 27  | Superhero |
| Pietro | 24  | Clown     |
| Flora  | 18  | Singer    |"#;

        let rows = parse(table_str).expect("Failed to parse table");
        let dumped = dump(&rows);

        // The regenerated table should parse back to the same rows.
        let reparsed = parse(&dumped).expect("Dumped table should re-parse");
        assert_eq!(rows, reparsed);
    }

    #[test]
    fn test_dump_empty_rows() {
        let rows: Vec<HashMap<String, String>> = Vec::new();
        assert_eq!(dump(&rows), String::new());
    }

    #[test]
    fn test_dump_missing_cell_renders_empty() {
        let mut row = HashMap::new();
        row.insert("name".to_string(), "Mira".to_string());
        row.insert("age".to_string(), "27".to_string());

        let mut other = HashMap::new();
        other.insert("job".to_string(), "Singer".to_string());

        let expected = "| age | job | name |\n|---|---|---|\n| 27 |  | Mira |\n|  | Singer |  |\n";
        assert_eq!(dump(&[row, other]), expected);
    }

    #[test]
    fn test_dump_columns_are_sorted() {
        let mut row = HashMap::new();
        row.insert("zebra".to_string(), "1".to_string());
        row.insert("alpha".to_string(), "2".to_string());

        let expected = "| alpha | zebra |\n|---|---|\n| 2 | 1 |\n";
        assert_eq!(dump(&[row]), expected);
    }
}
