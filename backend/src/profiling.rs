use std::collections::HashSet;

use chrono::NaiveDate;
use serde::Serialize;

use crate::csv_parser::CsvPreview;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ColumnProfile {
    pub index: usize,
    pub name: String,
    pub inferred_type: InferredColumnType,
    pub likely_meaning: InferredColumnMeaning,
    pub confidence: f32,
    pub non_empty_count: usize,
    pub blank_count: usize,
    pub unique_count: usize,
    pub sample_values: Vec<String>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferredColumnType {
    Empty,
    Date,
    Numeric,
    Boolean,
    Categorical,
    Text,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferredColumnMeaning {
    Unknown,
    Date,
    NumericTrend,
    FieldName,
    CropType,
    Category,
    Identifier,
}

#[derive(Debug, Default)]
struct ColumnSignals {
    non_empty_count: usize,
    blank_count: usize,
    date_count: usize,
    numeric_count: usize,
    boolean_count: usize,
    unique_values: HashSet<String>,
    sample_values: Vec<String>,
}

pub fn profile_columns(preview: &CsvPreview) -> Vec<ColumnProfile> {
    preview
        .columns
        .iter()
        .enumerate()
        .map(|(index, name)| profile_column(preview, index, name))
        .collect()
}

fn profile_column(preview: &CsvPreview, index: usize, name: &str) -> ColumnProfile {
    let signals = collect_signals(preview, index);
    let inferred_type = infer_type(&signals);
    let (likely_meaning, mut evidence) = infer_meaning(name, inferred_type, &signals);
    let confidence = confidence_for(inferred_type, likely_meaning, &signals);

    evidence.extend(type_evidence(inferred_type, &signals));

    ColumnProfile {
        index,
        name: name.to_owned(),
        inferred_type,
        likely_meaning,
        confidence,
        non_empty_count: signals.non_empty_count,
        blank_count: signals.blank_count,
        unique_count: signals.unique_values.len(),
        sample_values: signals.sample_values,
        evidence,
    }
}

fn collect_signals(preview: &CsvPreview, index: usize) -> ColumnSignals {
    let mut signals = ColumnSignals::default();

    for row in &preview.rows {
        let value = row.get(index).map(String::as_str).unwrap_or("").trim();
        if value.is_empty() {
            signals.blank_count += 1;
            continue;
        }

        signals.non_empty_count += 1;
        signals.unique_values.insert(value.to_ascii_lowercase());
        if signals.sample_values.len() < 3 {
            signals.sample_values.push(value.to_owned());
        }
        if parse_date(value).is_some() {
            signals.date_count += 1;
        }
        if parse_numeric(value).is_some() {
            signals.numeric_count += 1;
        }
        if parse_boolean(value).is_some() {
            signals.boolean_count += 1;
        }
    }

    signals
}

fn infer_type(signals: &ColumnSignals) -> InferredColumnType {
    if signals.non_empty_count == 0 {
        return InferredColumnType::Empty;
    }

    if ratio(signals.date_count, signals.non_empty_count) >= 0.75 {
        return InferredColumnType::Date;
    }
    if ratio(signals.numeric_count, signals.non_empty_count) >= 0.75 {
        return InferredColumnType::Numeric;
    }
    if ratio(signals.boolean_count, signals.non_empty_count) >= 0.75 {
        return InferredColumnType::Boolean;
    }
    if signals.unique_values.len() <= categorical_unique_limit(signals.non_empty_count) {
        return InferredColumnType::Categorical;
    }

    InferredColumnType::Text
}

fn infer_meaning(
    name: &str,
    inferred_type: InferredColumnType,
    signals: &ColumnSignals,
) -> (InferredColumnMeaning, Vec<String>) {
    let normalized = normalize_name(name);

    if contains_any(&normalized, &["date", "time", "year", "season"]) {
        return (
            InferredColumnMeaning::Date,
            vec![format!("Column name '{name}' suggests a date or time field.")],
        );
    }
    if contains_any(&normalized, &["field", "farm", "plot", "block", "paddock"]) {
        return (
            InferredColumnMeaning::FieldName,
            vec![format!("Column name '{name}' matches field or farm naming language.")],
        );
    }
    if contains_any(
        &normalized,
        &["crop", "commodity", "variety", "cultivar", "hybrid"],
    ) {
        return (
            InferredColumnMeaning::CropType,
            vec![format!("Column name '{name}' matches crop classification language.")],
        );
    }
    if contains_any(&normalized, &["id", "code", "number", "no"]) {
        return (
            InferredColumnMeaning::Identifier,
            vec![format!("Column name '{name}' suggests an identifier.")],
        );
    }
    if inferred_type == InferredColumnType::Numeric
        && contains_any(
            &normalized,
            &[
                "yield", "rate", "acre", "acres", "area", "moisture", "price", "cost", "revenue",
                "total", "average", "avg", "count", "score", "index", "tons", "bushel",
            ],
        )
    {
        return (
            InferredColumnMeaning::NumericTrend,
            vec![format!(
                "Numeric column name '{name}' matches trend or measurement language."
            )],
        );
    }
    if inferred_type == InferredColumnType::Date {
        return (
            InferredColumnMeaning::Date,
            vec!["Most sampled values parsed as dates.".to_owned()],
        );
    }
    if inferred_type == InferredColumnType::Categorical {
        return (
            InferredColumnMeaning::Category,
            vec![format!(
                "{} distinct sampled values fit a categorical field.",
                signals.unique_values.len()
            )],
        );
    }

    (InferredColumnMeaning::Unknown, Vec::new())
}

fn type_evidence(inferred_type: InferredColumnType, signals: &ColumnSignals) -> Vec<String> {
    match inferred_type {
        InferredColumnType::Empty => vec!["No non-empty sampled values were found.".to_owned()],
        InferredColumnType::Date => vec![format!(
            "{} of {} sampled values parsed as dates.",
            signals.date_count, signals.non_empty_count
        )],
        InferredColumnType::Numeric => vec![format!(
            "{} of {} sampled values parsed as numbers.",
            signals.numeric_count, signals.non_empty_count
        )],
        InferredColumnType::Boolean => vec![format!(
            "{} of {} sampled values parsed as booleans.",
            signals.boolean_count, signals.non_empty_count
        )],
        InferredColumnType::Categorical => vec![format!(
            "{} unique sampled values across {} non-empty rows.",
            signals.unique_values.len(),
            signals.non_empty_count
        )],
        InferredColumnType::Text => vec![format!(
            "{} unique sampled values did not meet date, numeric, or category thresholds.",
            signals.unique_values.len()
        )],
    }
}

fn confidence_for(
    inferred_type: InferredColumnType,
    likely_meaning: InferredColumnMeaning,
    signals: &ColumnSignals,
) -> f32 {
    if signals.non_empty_count == 0 {
        return 0.0;
    }

    let type_confidence = match inferred_type {
        InferredColumnType::Date => ratio(signals.date_count, signals.non_empty_count),
        InferredColumnType::Numeric => ratio(signals.numeric_count, signals.non_empty_count),
        InferredColumnType::Boolean => ratio(signals.boolean_count, signals.non_empty_count),
        InferredColumnType::Categorical => {
            1.0 - ratio(signals.unique_values.len(), signals.non_empty_count).min(0.8)
        }
        InferredColumnType::Text => 0.55,
        InferredColumnType::Empty => 0.0,
    };
    let meaning_bonus = if likely_meaning == InferredColumnMeaning::Unknown {
        0.0
    } else {
        0.15
    };

    (type_confidence + meaning_bonus).clamp(0.0, 0.99)
}

fn categorical_unique_limit(non_empty_count: usize) -> usize {
    if non_empty_count <= 4 {
        non_empty_count
    } else {
        (non_empty_count / 2).clamp(3, 20)
    }
}

fn normalize_name(name: &str) -> String {
    name.to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    value.split_whitespace().any(|part| needles.contains(&part))
}

fn ratio(count: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        count as f32 / total as f32
    }
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    let value = value.trim();
    let formats = [
        "%Y-%m-%d", "%m/%d/%Y", "%m/%d/%y", "%d/%m/%Y", "%d/%m/%y", "%Y/%m/%d", "%b %d %Y",
        "%B %d %Y",
    ];

    for format in formats {
        if let Ok(date) = NaiveDate::parse_from_str(value, format) {
            return Some(date);
        }
    }

    if value.len() == 4 {
        if let Ok(year) = value.parse::<i32>() {
            if (1900..=2200).contains(&year) {
                return NaiveDate::from_ymd_opt(year, 1, 1);
            }
        }
    }

    None
}

fn parse_numeric(value: &str) -> Option<f64> {
    let mut cleaned = value
        .trim()
        .trim_start_matches('$')
        .trim_end_matches('%')
        .replace(',', "");
    let is_parenthesized = cleaned.starts_with('(') && cleaned.ends_with(')');

    if is_parenthesized {
        cleaned = cleaned
            .trim_start_matches('(')
            .trim_end_matches(')')
            .to_owned();
    }

    cleaned.parse::<f64>().ok()
}

fn parse_boolean(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "t" | "yes" | "y" | "1" => Some(true),
        "false" | "f" | "no" | "n" | "0" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::csv_parser::parse_csv_preview;

    use super::{profile_columns, InferredColumnMeaning, InferredColumnType};

    #[test]
    fn detects_dates_numeric_trends_and_crop_categories() {
        let preview = parse_csv_preview(
            b"planting_date,yield_bu,crop_type\n2026-04-01,180,Corn\n2026-04-08,176,Soybean\n2026-04-15,190,Corn\n",
            25,
        )
        .expect("CSV should parse");
        let profiles = profile_columns(&preview);

        assert_eq!(profiles[0].inferred_type, InferredColumnType::Date);
        assert_eq!(profiles[0].likely_meaning, InferredColumnMeaning::Date);
        assert_eq!(profiles[1].inferred_type, InferredColumnType::Numeric);
        assert_eq!(
            profiles[1].likely_meaning,
            InferredColumnMeaning::NumericTrend
        );
        assert_eq!(profiles[2].inferred_type, InferredColumnType::Categorical);
        assert_eq!(profiles[2].likely_meaning, InferredColumnMeaning::CropType);
    }

    #[test]
    fn detects_field_name_from_header() {
        let preview =
            parse_csv_preview(b"field_name,notes\nNorth 40,Needs lime\nSouth 80,\n", 25)
                .expect("CSV should parse");
        let profiles = profile_columns(&preview);

        assert_eq!(profiles[0].likely_meaning, InferredColumnMeaning::FieldName);
        assert!(profiles[0].evidence.iter().any(|item| item.contains("field")));
    }
}
