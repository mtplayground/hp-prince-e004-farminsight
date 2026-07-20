use csv::{ByteRecord, ReaderBuilder, Trim};
use serde::Serialize;
use std::collections::HashMap;

const SNIFF_BYTES: usize = 16 * 1024;
const DELIMITER_CANDIDATES: [u8; 4] = [b',', b';', b'\t', b'|'];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CsvPreview {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    pub column_count: usize,
    pub delimiter: char,
    pub truncated: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CsvParseError {
    #[error("CSV file has no parseable rows")]
    Empty,
}

pub fn parse_csv_preview(
    bytes: &[u8],
    max_preview_rows: usize,
) -> Result<CsvPreview, CsvParseError> {
    let delimiter = sniff_delimiter(bytes);
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(Trim::All)
        .delimiter(delimiter)
        .from_reader(bytes);
    let mut records = Vec::new();
    let mut warnings = Vec::new();

    for result in reader.byte_records() {
        match result {
            Ok(record) if !is_empty_record(&record) => {
                records.push(byte_record_to_fields(&record));
            }
            Ok(_) => {}
            Err(error) => {
                warnings.push(format!(
                    "CSV parser recovered with line fallback after: {}",
                    error
                ));
                return parse_lossy_lines(bytes, delimiter, max_preview_rows, warnings);
            }
        }
    }

    build_preview(records, delimiter, max_preview_rows, warnings)
}

pub fn parse_csv_metadata(
    bytes: &[u8],
) -> Result<(Option<i64>, Option<i32>, Vec<String>), CsvParseError> {
    let preview = parse_csv_preview(bytes, 0)?;
    let row_count = i64::try_from(preview.row_count).ok();
    let column_count = i32::try_from(preview.column_count).ok();

    Ok((row_count, column_count, preview.columns))
}

fn parse_lossy_lines(
    bytes: &[u8],
    delimiter: u8,
    max_preview_rows: usize,
    mut warnings: Vec<String>,
) -> Result<CsvPreview, CsvParseError> {
    let text = String::from_utf8_lossy(bytes);
    let records = text
        .lines()
        .map(|line| split_lossy_line(line, delimiter as char))
        .filter(|record| record.iter().any(|field| !field.trim().is_empty()))
        .collect::<Vec<_>>();

    warnings.push(
        "Used lossy line-based parsing for malformed CSV quoting or encoding.".to_owned(),
    );
    build_preview(records, delimiter, max_preview_rows, warnings)
}

fn build_preview(
    mut records: Vec<Vec<String>>,
    delimiter: u8,
    max_preview_rows: usize,
    mut warnings: Vec<String>,
) -> Result<CsvPreview, CsvParseError> {
    if records.is_empty() {
        return Err(CsvParseError::Empty);
    }

    let header = records.remove(0);
    let mut columns = normalize_columns(header);
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row_count = 0usize;
    let mut saw_short_row = false;
    let mut saw_wide_row = false;

    for record in records {
        row_count += 1;

        if record.len() > columns.len() {
            saw_wide_row = true;
            let start = columns.len();
            for index in start..record.len() {
                columns.push(format!("extra_column_{}", index + 1));
            }
            for row in &mut rows {
                row.resize(columns.len(), String::new());
            }
        } else if record.len() < columns.len() {
            saw_short_row = true;
        }

        if rows.len() < max_preview_rows {
            let mut normalized = record;
            normalized.resize(columns.len(), String::new());
            rows.push(normalized);
        }
    }

    if saw_short_row {
        warnings.push(
            "Some rows had fewer fields than the header and were padded with blanks.".to_owned(),
        );
    }
    if saw_wide_row {
        warnings.push(
            "Some rows had extra fields; preview columns were added for the widest rows.".to_owned(),
        );
    }

    Ok(CsvPreview {
        column_count: columns.len(),
        columns,
        rows,
        row_count,
        delimiter: delimiter as char,
        truncated: row_count > max_preview_rows,
        warnings,
    })
}

fn sniff_delimiter(bytes: &[u8]) -> u8 {
    let text = String::from_utf8_lossy(&bytes[..bytes.len().min(SNIFF_BYTES)]);
    let mut scores = HashMap::new();

    for line in text.lines().filter(|line| !line.trim().is_empty()).take(10) {
        for delimiter in DELIMITER_CANDIDATES {
            let count = count_delimiter_outside_quotes(line, delimiter as char);
            *scores.entry(delimiter).or_insert(0usize) += count;
        }
    }

    DELIMITER_CANDIDATES
        .into_iter()
        .max_by_key(|delimiter| scores.get(delimiter).copied().unwrap_or_default())
        .unwrap_or(b',')
}

fn count_delimiter_outside_quotes(line: &str, delimiter: char) -> usize {
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    let mut count = 0usize;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                let _ = chars.next();
            }
            '"' => in_quotes = !in_quotes,
            value if value == delimiter && !in_quotes => count += 1,
            _ => {}
        }
    }

    count
}

fn byte_record_to_fields(record: &ByteRecord) -> Vec<String> {
    record
        .iter()
        .map(|field| String::from_utf8_lossy(field).trim().to_owned())
        .collect()
}

fn is_empty_record(record: &ByteRecord) -> bool {
    record
        .iter()
        .all(|field| String::from_utf8_lossy(field).trim().is_empty())
}

fn normalize_columns(raw_columns: Vec<String>) -> Vec<String> {
    let mut seen = HashMap::new();

    raw_columns
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let base = clean_header(&value, index);
            let seen_count = seen.entry(base.clone()).or_insert(0usize);
            *seen_count += 1;

            if *seen_count == 1 {
                base
            } else {
                format!("{}_{}", base, seen_count)
            }
        })
        .collect()
}

fn clean_header(value: &str, index: usize) -> String {
    let value = value
        .trim()
        .trim_start_matches('\u{feff}')
        .trim_matches('"')
        .trim_matches('\'')
        .trim();

    if value.is_empty() {
        format!("column_{}", index + 1)
    } else {
        value.to_owned()
    }
}

fn split_lossy_line(line: &str, delimiter: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                field.push('"');
                let _ = chars.next();
            }
            '"' => in_quotes = !in_quotes,
            value if value == delimiter && !in_quotes => {
                fields.push(field.trim().trim_matches('"').to_owned());
                field.clear();
            }
            value => field.push(value),
        }
    }

    fields.push(field.trim().trim_matches('"').to_owned());
    fields
}

#[cfg(test)]
mod tests {
    use super::parse_csv_preview;

    #[test]
    fn pads_short_rows_and_adds_extra_columns() {
        let preview = parse_csv_preview(b"name,acres\nNorth,10,extra\nSouth\n", 10)
            .expect("preview should parse");

        assert_eq!(preview.columns, vec!["name", "acres", "extra_column_3"]);
        assert_eq!(preview.rows[0], vec!["North", "10", "extra"]);
        assert_eq!(preview.rows[1], vec!["South", "", ""]);
        assert_eq!(preview.row_count, 2);
        assert_eq!(preview.column_count, 3);
        assert_eq!(preview.warnings.len(), 2);
    }

    #[test]
    fn detects_semicolon_delimiter() {
        let preview =
            parse_csv_preview("field;yield\nwest;42\n".as_bytes(), 5).expect("preview should parse");

        assert_eq!(preview.delimiter, ';');
        assert_eq!(preview.columns, vec!["field", "yield"]);
        assert_eq!(preview.rows, vec![vec!["west", "42"]]);
    }

    #[test]
    fn normalizes_blank_and_duplicate_headers() {
        let preview =
            parse_csv_preview(b",name,name\n1,A,B\n", 5).expect("preview should parse");

        assert_eq!(preview.columns, vec!["column_1", "name", "name_2"]);
    }
}
