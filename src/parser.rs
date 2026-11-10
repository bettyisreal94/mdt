use std::collections::HashMap;

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
    fn test_parse_missing_separator_fails() {
        let table_str = r#"| name | age |
| Mira | 27  |"#;

        let result = parse(table_str).expect_err("Should fail without a separator row");
        assert!(result.contains("Missing separator row"));
    }
}
