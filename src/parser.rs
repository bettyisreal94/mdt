use std::collections::HashMap;
use std::fmt;

/// Represents a single row in the parsed table as a map of column names to values
#[derive(Debug, Clone)]
pub struct Row {
    pub columns: HashMap<String, String>,
}

impl Row {
    pub fn new() -> Self {
        Row {
            columns: HashMap::new(),
        }
    }

    pub fn with_column(&mut self, name: &str, value: &str) {
        self.columns.insert(name.to_string(), value.to_string());
    }
}

/// Represents the entire parsed markdown table with header information  
#[derive(Debug, Clone)]
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Row>,
}

impl Table {
    pub fn new(headers: Vec<String>) -> Self {
        Table { headers, rows: Vec::new() }
    }

    pub fn with_rows(&mut self, rows: Vec<Row>) {
        self.rows = rows;
    }

    pub fn to_rows(&self) -> Vec<Row> {
        self.rows.clone()
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.headers.is_empty() || self.rows.is_empty() {
            return Ok(());
        }

        let mut col_widths = vec![0usize; self.headers.len()];
        
        for row in &self.rows {
            for (i, value) in row.columns.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(value.len());
                }
            }
        }

        for _ in 0..col_widths.len() {
            col_widths[_] = col_widths[_].max(4);
        }

        // Header row
        write!(f, "|")?;
        for (i, header) in self.headers.iter().enumerate() {
            let width = col_widths[i];
            let padding = if i == 0 || i == self.headers.len() - 1 { 2 } else { 1 };
            write!(f, " {:header:>width$} |", padding, header = &header[..], width = width)?;
        }
        writeln!(f)

        // Separator row
        write!(f, "|")?;
        for _ in 0..col_widths.len() {
            write!(f, " --- |")?;
        }
        writeln!(f)

        // Data rows
        for row in &self.rows {
            write!(f, "|")?;
            for (i, value) in row.columns.iter().enumerate() {
                let width = col_widths[i];
                let padding = if i == 0 || i == self.headers.len() - 1 { 2 } else { 1 };
                write!(f, " {:value:>width$} |", padding, value = &value[..], width = width)?;
            }
            writeln!(f)
        }

        Ok(())
    }
}

/// Parse an extended markdown table string into a Table struct.
/// Extension lines (starting with ":") are ignored for now.
/// Returns Ok(Table) if successful, or Err(String) if parsing fails.
pub fn parse(table_str: &str) -> Result<Table, String> {
    let lines: Vec<&str> = table_str.lines().collect();

    // Need at least header and separator line
    if lines.len() < 2 {
        return Err("Table must have at least a header row and separator".to_string());
    }

    // Parse header row (first non-empty line)
    let header_line = find_first_non_empty(&lines);
    let headers = parse_header_row(header_line)?;

    // Collect data rows, skipping header and separator lines
    let mut rows: Vec<Row> = Vec::new();
    
    for line in &lines {
        if line.trim().is_empty() {
            continue;  // Skip empty lines
        }

        // Check if this is an extension line (starts with ":")
        if line.starts_with(":") {
            continue;  // Ignore extension lines for now
        }

        // Parse data row
        let row = parse_data_row(line)?;
        rows.push(row);
    }

    Ok(Table::new(headers).with_rows(rows))
}

fn find_first_non_empty(lines: &[&str]) -> usize {
    for (i, line) in lines.iter().enumerate() {
        if !line.trim().is_empty() {
            return i;
        }
    }
    0  // Default to first line
}

fn parse_header_row(line: &str) -> Result<Vec<String>, String> {
    let clean = line.trim_start_matches("|").trim_end_matches("|");
    
    if clean.is_empty() {
        return Err("Empty header row".to_string());
    }

    let fields: Vec<String> = clean.split("|").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    
    if fields.is_empty() {
        return Err("Header row must have at least one column".to_string());
    }

    Ok(fields)
}

fn parse_data_row(line: &str) -> Result<Row, String> {
    let mut row = Row::new();
    
    let clean = line.trim_start_matches("|").trim_end_matches("|");
    
    if clean.is_empty() {
        return Err("Empty data row".to_string());
    }

    let fields: Vec<&str> = clean.split("|").map(|s| s.trim()).collect();
    
    if fields.is_empty() {
        return Err("Data row must have at least one value".to_string());
    }

    // Assign fields to columns by name
    for (i, field) in fields.iter().enumerate() {
        let col_name = format!("column_{}", i);
        if !field.is_empty() {
            row.with_column(&col_name, field);
        } else {
            row.with_column(&col_name, "");
        }
    }

    Ok(row)
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

        let table = parse(table_str).expect("Failed to parse basic table");
        
        assert_eq!(table.headers.len(), 3);
        assert_eq!(table.headers[0], "name");
        assert_eq!(table.headers[1], "age");
        assert_eq!(table.headers[2], "job");
        
        assert_eq!(table.rows.len(), 3);
        
        let first_row = &table.rows[0];
        assert_eq!(first_row.columns.get("column_0").unwrap(), "Mira");
        assert_eq!(first_row.columns.get("column_1").unwrap(), "27");
        assert_eq!(first_row.columns.get("column_2").unwrap(), "Superhero");
    }

    #[test]
    fn test_parse_with_extension_lines() {
        let table_str = r#"| name   | age | job       |
|--------|-----|-----------|
| Mira   | 27  | Superhero |
: title = (t, r, i) => `${r["name"]}, the ${r["job"]}`"#;

        let table = parse(table_str).expect("Failed to parse table with extension lines");
        
        // Extension lines should be ignored for now
        assert_eq!(table.headers.len(), 3);
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn test_parse_empty_table() {
        let result = parse("").expect_err("Should fail on empty string");
        assert!(result.contains("Table must have at least"));
    }

    #[test]
    fn test_parse_invalid_table() {
        let result = parse("no pipe characters here").expect_err("Should fail on invalid table");
        assert!(result.contains("Header row must have at least one column"));
    }

    #[test]
    fn test_parse_with_empty_cell() {
        let table_str = r#"| name   | age | job       |
|--------|-----|-----------|
| Mira   | 27  | Superhero |
| Pietro |     | Clown     |"#;

        let table = parse(table_str).expect("Failed to parse table with empty cell");
        
        let second_row = &table.rows[0];
        assert!(second_row.columns.get("column_1").is_some());
    }
}

