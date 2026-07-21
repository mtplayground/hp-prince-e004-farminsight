use std::collections::HashMap;

use chrono::NaiveDate;
use serde::Serialize;

use crate::{
    csv_parser::CsvPreview,
    profiling::{ColumnProfile, InferredColumnMeaning, InferredColumnType},
};

const MAX_INSIGHTS: usize = 5;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Insight {
    pub kind: InsightKind,
    pub title: String,
    pub summary: String,
    pub evidence: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InsightKind {
    Trend,
    Comparison,
    CategoryMix,
    DataQuality,
    DatasetShape,
}

pub fn generate_insights(preview: &CsvPreview, profiles: &[ColumnProfile]) -> Vec<Insight> {
    let mut insights = Vec::new();

    if let Some(insight) = dataset_shape_insight(preview, profiles) {
        insights.push(insight);
    }
    insights.extend(trend_insights(preview, profiles));
    insights.extend(comparison_insights(preview, profiles));
    insights.extend(category_mix_insights(preview, profiles));
    insights.extend(data_quality_insights(preview, profiles));

    insights.truncate(MAX_INSIGHTS);
    insights
}

fn dataset_shape_insight(preview: &CsvPreview, profiles: &[ColumnProfile]) -> Option<Insight> {
    if preview.column_count == 0 {
        return None;
    }

    let numeric_count = profiles
        .iter()
        .filter(|profile| profile.inferred_type == InferredColumnType::Numeric)
        .count();
    let category_count = profiles
        .iter()
        .filter(|profile| {
            matches!(
                profile.likely_meaning,
                InferredColumnMeaning::CropType
                    | InferredColumnMeaning::FieldName
                    | InferredColumnMeaning::Category
            )
        })
        .count();

    let mut evidence = vec![format!(
        "Parser used '{}' as the delimiter and kept {} preview rows.",
        preview.delimiter,
        preview.rows.len()
    )];
    evidence.extend(preview.warnings.iter().take(2).cloned());

    if preview.row_count == 0 {
        return Some(Insight {
            kind: InsightKind::DataQuality,
            title: "Dataset needs data rows before insights".to_owned(),
            summary: format!(
                "I found {} column{} but no data rows, so trends, comparisons, and charts are not reliable yet.",
                preview.column_count,
                plural_suffix(preview.column_count)
            ),
            evidence,
            confidence: 0.35,
        });
    }

    let unstructured = numeric_count == 0 && category_count == 0;
    let title = if unstructured {
        "Dataset structure is limited"
    } else {
        "Dataset is ready for a quick scan"
    };
    let summary = if unstructured {
        format!(
            "I found {} rows and {} columns, but no clear numeric measures or grouping fields. Treat summaries as data-quality notes until the columns are cleaned.",
            preview.row_count, preview.column_count
        )
    } else {
        format!(
            "I found {} rows and {} columns, including {} numeric measure{} and {} likely grouping field{}.",
            preview.row_count,
            preview.column_count,
            numeric_count,
            plural_suffix(numeric_count),
            category_count,
            plural_suffix(category_count)
        )
    };

    Some(Insight {
        kind: if unstructured {
            InsightKind::DataQuality
        } else {
            InsightKind::DatasetShape
        },
        title: title.to_owned(),
        summary,
        evidence,
        confidence: if unstructured {
            0.55
        } else if preview.truncated || !preview.warnings.is_empty() {
            0.72
        } else {
            0.82
        },
    })
}

fn trend_insights(preview: &CsvPreview, profiles: &[ColumnProfile]) -> Vec<Insight> {
    let Some(date_column) = profiles.iter().find(|profile| {
        profile.likely_meaning == InferredColumnMeaning::Date
            || profile.inferred_type == InferredColumnType::Date
    }) else {
        return Vec::new();
    };

    profiles
        .iter()
        .filter(|profile| {
            profile.inferred_type == InferredColumnType::Numeric
                && matches!(
                    profile.likely_meaning,
                    InferredColumnMeaning::NumericTrend | InferredColumnMeaning::Unknown
                )
        })
        .filter_map(|numeric_column| trend_for_column(preview, date_column, numeric_column))
        .take(2)
        .collect()
}

fn trend_for_column(
    preview: &CsvPreview,
    date_column: &ColumnProfile,
    numeric_column: &ColumnProfile,
) -> Option<Insight> {
    let mut values_by_date: HashMap<NaiveDate, RunningStat> = HashMap::new();

    for row in &preview.rows {
        let Some(date) = row
            .get(date_column.index)
            .and_then(|value| parse_date(value.as_str()))
        else {
            continue;
        };
        let Some(value) = row
            .get(numeric_column.index)
            .and_then(|value| parse_number(value.as_str()))
        else {
            continue;
        };

        values_by_date.entry(date).or_default().push(value);
    }

    if values_by_date.len() < 2 {
        return None;
    }

    let mut values = values_by_date
        .into_iter()
        .map(|(date, stat)| (date, stat.average(), stat.count))
        .collect::<Vec<_>>();
    values.sort_by_key(|(date, _, _)| *date);
    let (first_date, first_value, first_count) = values.first().copied()?;
    let (last_date, last_value, last_count) = values.last().copied()?;
    if first_date == last_date || nearly_equal(first_value, last_value) {
        return None;
    }

    let delta = last_value - first_value;
    let direction = if delta > 0.0 { "increased" } else { "decreased" };
    let percent_text = percent_change(first_value, last_value)
        .map(|value| format!(", a {} change", format_percent(value)))
        .unwrap_or_default();

    Some(Insight {
        kind: InsightKind::Trend,
        title: format!("{} {} over time", numeric_column.name, direction),
        summary: format!(
            "{} {} from {} to {} between {} and {}{}.",
            numeric_column.name,
            direction,
            format_number(first_value),
            format_number(last_value),
            first_date,
            last_date,
            percent_text
        ),
        evidence: vec![format!(
            "Compared average values from {} row{} at the start and {} row{} at the end using {} as the date field.",
            first_count,
            plural_suffix(first_count),
            last_count,
            plural_suffix(last_count),
            date_column.name
        )],
        confidence: (numeric_column.confidence + date_column.confidence)
            .mul_add(0.5, 0.0)
            .min(0.95),
    })
}

fn comparison_insights(preview: &CsvPreview, profiles: &[ColumnProfile]) -> Vec<Insight> {
    let Some(group_column) = profiles.iter().find(|profile| {
        matches!(
            profile.likely_meaning,
            InferredColumnMeaning::CropType
                | InferredColumnMeaning::FieldName
                | InferredColumnMeaning::Category
        )
    }) else {
        return Vec::new();
    };

    profiles
        .iter()
        .filter(|profile| profile.inferred_type == InferredColumnType::Numeric)
        .filter_map(|numeric_column| {
            comparison_for_column(preview, group_column, numeric_column)
        })
        .take(2)
        .collect()
}

fn comparison_for_column(
    preview: &CsvPreview,
    group_column: &ColumnProfile,
    numeric_column: &ColumnProfile,
) -> Option<Insight> {
    let mut groups: HashMap<String, RunningStat> = HashMap::new();

    for row in &preview.rows {
        let Some(group) = row
            .get(group_column.index)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let Some(value) = row
            .get(numeric_column.index)
            .and_then(|value| parse_number(value.as_str()))
        else {
            continue;
        };

        groups.entry(group.to_owned()).or_default().push(value);
    }

    if groups.len() < 2 {
        return None;
    }

    let mut ranked = groups
        .into_iter()
        .filter(|(_, stat)| stat.count > 0)
        .map(|(name, stat)| (name, stat.average(), stat.count))
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1));

    let (top_name, top_average, top_count) = ranked.first()?;
    let (bottom_name, bottom_average, bottom_count) = ranked.last()?;
    if top_name == bottom_name || nearly_equal(*top_average, *bottom_average) {
        return None;
    }

    Some(Insight {
        kind: InsightKind::Comparison,
        title: format!("{top_name} leads on {}", numeric_column.name),
        summary: format!(
            "{top_name} has the highest average {} at {}, compared with {} for {bottom_name}.",
            numeric_column.name,
            format_number(*top_average),
            format_number(*bottom_average)
        ),
        evidence: vec![format!(
            "Compared {} groups using {} rows for {top_name} and {} rows for {bottom_name}.",
            ranked.len(),
            top_count,
            bottom_count
        )],
        confidence: (numeric_column.confidence + group_column.confidence)
            .mul_add(0.45, 0.05)
            .min(0.9),
    })
}

fn category_mix_insights(preview: &CsvPreview, profiles: &[ColumnProfile]) -> Vec<Insight> {
    profiles
        .iter()
        .filter(|profile| {
            matches!(
                profile.likely_meaning,
                InferredColumnMeaning::CropType
                    | InferredColumnMeaning::FieldName
                    | InferredColumnMeaning::Category
            )
        })
        .filter_map(|profile| category_mix_for_column(preview, profile))
        .take(1)
        .collect()
}

fn category_mix_for_column(preview: &CsvPreview, profile: &ColumnProfile) -> Option<Insight> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for row in &preview.rows {
        if let Some(value) = row
            .get(profile.index)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            *counts.entry(value.to_owned()).or_default() += 1;
        }
    }

    if counts.len() < 2 {
        return None;
    }

    let (top_value, top_count) = counts
        .iter()
        .max_by_key(|(_, count)| **count)
        .map(|(value, count)| (value.as_str(), *count))?;
    let total = counts.values().sum::<usize>();

    Some(Insight {
        kind: InsightKind::CategoryMix,
        title: format!("{} is concentrated around {top_value}", profile.name),
        summary: format!(
            "{top_value} appears most often in {}, showing up in {} of {} filled rows.",
            profile.name, top_count, total
        ),
        evidence: vec![format!(
            "{} distinct values were sampled for {}.",
            counts.len(),
            profile.name
        )],
        confidence: profile.confidence.min(0.88),
    })
}

fn data_quality_insights(preview: &CsvPreview, profiles: &[ColumnProfile]) -> Vec<Insight> {
    let mut insights = Vec::new();

    if !preview.warnings.is_empty() {
        insights.push(Insight {
            kind: InsightKind::DataQuality,
            title: "Parser recovered messy CSV structure".to_owned(),
            summary: "Some rows needed parser recovery or normalization, so read trend and comparison findings as directional until the source CSV is cleaned.".to_owned(),
            evidence: preview.warnings.iter().take(3).cloned().collect(),
            confidence: 0.76,
        });
    }

    if let Some(profile) = profiles
        .iter()
        .filter(|profile| profile.blank_count > 0)
        .max_by_key(|profile| profile.blank_count)
    {
        let total = profile.blank_count + profile.non_empty_count;
        if total > 0 {
            insights.push(Insight {
                kind: InsightKind::DataQuality,
                title: format!("{} has missing values", profile.name),
                summary: format!(
                    "{} is blank in {} of {} preview rows, so treat summaries using that column as directional.",
                    profile.name, profile.blank_count, total
                ),
                evidence: vec![format!(
                    "{} non-empty values were found for {}.",
                    profile.non_empty_count, profile.name
                )],
                confidence: 0.86,
            });
        }
    }

    if preview.truncated {
        insights.push(Insight {
            kind: InsightKind::DataQuality,
            title: "Preview is based on a sample".to_owned(),
            summary: format!(
                "The file has {} rows, and insights were generated from the first {} parsed rows.",
                preview.row_count,
                preview.rows.len()
            ),
            evidence: vec![
                "Use the full dataset for final decisions when later processing is available."
                    .to_owned(),
            ],
            confidence: 0.8,
        });
    }

    insights
}

#[derive(Default)]
struct RunningStat {
    sum: f64,
    count: usize,
}

impl RunningStat {
    fn push(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
    }

    fn average(&self) -> f64 {
        self.sum / self.count as f64
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

fn parse_number(value: &str) -> Option<f64> {
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

fn percent_change(first: f64, last: f64) -> Option<f64> {
    if nearly_equal(first, 0.0) {
        None
    } else {
        Some((last - first) / first * 100.0)
    }
}

fn nearly_equal(left: f64, right: f64) -> bool {
    (left - right).abs() < f64::EPSILON
}

fn format_percent(value: f64) -> String {
    format!("{value:.1}%")
}

fn format_number(value: f64) -> String {
    if nearly_equal(value.fract(), 0.0) {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

fn plural_suffix(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        csv_parser::parse_csv_preview,
        profiling::{profile_columns, InferredColumnMeaning},
    };

    use super::{generate_insights, InsightKind};

    #[test]
    fn summarizes_trends_and_category_comparisons() {
        let preview = parse_csv_preview(
            b"season,crop_type,yield_bu\n2024,Corn,180\n2025,Corn,195\n2024,Soybean,52\n2025,Soybean,58\n",
            25,
        )
        .expect("CSV should parse");
        let profiles = profile_columns(&preview);
        let insights = generate_insights(&preview, &profiles);

        assert!(insights.iter().any(|insight| {
            insight.kind == InsightKind::Trend
                && insight.summary.contains("yield_bu increased")
        }));
        assert!(insights.iter().any(|insight| {
            insight.kind == InsightKind::Comparison
                && insight.summary.contains("highest average yield_bu")
        }));
    }

    #[test]
    fn reports_missing_values_in_plain_language() {
        let preview = parse_csv_preview(
            b"field_name,crop_type,yield_bu\nNorth,Corn,180\nSouth,,172\nEast,Soybean,\n",
            25,
        )
        .expect("CSV should parse");
        let profiles = profile_columns(&preview);
        let insights = generate_insights(&preview, &profiles);

        assert!(profiles
            .iter()
            .any(|profile| profile.likely_meaning == InferredColumnMeaning::FieldName));
        assert!(insights
            .iter()
            .any(|insight| insight.kind == InsightKind::DataQuality
                && insight.summary.contains("blank")));
    }

    #[test]
    fn warns_instead_of_overstating_header_only_csvs() {
        let preview = parse_csv_preview(b"field,season,yield\n", 25).expect("CSV should parse");
        let profiles = profile_columns(&preview);
        let insights = generate_insights(&preview, &profiles);

        let first = insights.first().expect("quality insight should be present");
        assert_eq!(first.kind, InsightKind::DataQuality);
        assert!(first.title.contains("needs data rows"));
        assert!(first.summary.contains("not reliable"));
    }

    #[test]
    fn surfaces_parser_recovery_notes_as_data_quality() {
        let preview =
            parse_csv_preview(b"field,yield\nNorth,180\nSouth\n", 25).expect("CSV should parse");
        let profiles = profile_columns(&preview);
        let insights = generate_insights(&preview, &profiles);

        assert!(insights.iter().any(|insight| {
            insight.kind == InsightKind::DataQuality
                && insight.title.contains("messy CSV structure")
        }));
    }
}
