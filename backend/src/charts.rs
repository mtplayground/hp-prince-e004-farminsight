use std::collections::HashMap;

use serde::Serialize;

use crate::{
    csv_parser::CsvPreview,
    profiling::{ColumnProfile, InferredColumnMeaning, InferredColumnType},
};

const MAX_CHART_SPECS: usize = 4;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ChartSpec {
    pub id: String,
    pub chart_type: ChartType,
    pub title: String,
    pub rationale: String,
    pub x: ChartField,
    pub y: ChartField,
    pub series: Option<ChartField>,
    pub aggregation: Option<Aggregation>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    Line,
    Bar,
    Scatter,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ChartField {
    pub index: usize,
    pub name: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Aggregation {
    Average,
}

pub fn select_chart_specs(preview: &CsvPreview, profiles: &[ColumnProfile]) -> Vec<ChartSpec> {
    let mut specs = Vec::new();

    if let Some(spec) = trend_line_spec(profiles) {
        specs.push(spec);
    }
    if let Some(spec) = category_comparison_spec(preview, profiles) {
        specs.push(spec);
    }
    if let Some(spec) = correlation_spec(preview, profiles) {
        specs.push(spec);
    }

    specs.truncate(MAX_CHART_SPECS);
    specs
}

fn trend_line_spec(profiles: &[ColumnProfile]) -> Option<ChartSpec> {
    let date_column = profiles.iter().find(|profile| {
        profile.likely_meaning == InferredColumnMeaning::Date
            || profile.inferred_type == InferredColumnType::Date
    })?;
    let numeric_column = profiles
        .iter()
        .filter(|profile| profile.inferred_type == InferredColumnType::Numeric)
        .max_by(|left, right| left.confidence.total_cmp(&right.confidence))?;
    let series = profiles.iter().find(|profile| {
        matches!(
            profile.likely_meaning,
            InferredColumnMeaning::CropType
                | InferredColumnMeaning::FieldName
                | InferredColumnMeaning::Category
        ) && profile.unique_count <= 12
    });

    let confidence = if let Some(series_column) = series {
        combined_confidence(&[date_column, numeric_column, series_column])
    } else {
        combined_confidence(&[date_column, numeric_column])
    };

    Some(ChartSpec {
        id: format!("line-{}-{}", date_column.index, numeric_column.index),
        chart_type: ChartType::Line,
        title: format!("{} over {}", numeric_column.name, date_column.name),
        rationale: format!(
            "{} looks like a time field and {} is numeric, so a trend line can show season-over-season movement.",
            date_column.name, numeric_column.name
        ),
        x: field_ref(date_column),
        y: field_ref(numeric_column),
        series: series.map(field_ref),
        aggregation: Some(Aggregation::Average),
        confidence,
    })
}

fn category_comparison_spec(
    preview: &CsvPreview,
    profiles: &[ColumnProfile],
) -> Option<ChartSpec> {
    let category_column = profiles
        .iter()
        .filter(|profile| {
            matches!(
                profile.likely_meaning,
                InferredColumnMeaning::CropType
                    | InferredColumnMeaning::FieldName
                    | InferredColumnMeaning::Category
            )
        })
        .filter(|profile| profile.unique_count >= 2 && profile.unique_count <= 20)
        .max_by(|left, right| left.confidence.total_cmp(&right.confidence))?;
    let numeric_column = profiles
        .iter()
        .filter(|profile| profile.inferred_type == InferredColumnType::Numeric)
        .max_by(|left, right| left.confidence.total_cmp(&right.confidence))?;

    if grouped_numeric_values(preview, category_column, numeric_column).len() < 2 {
        return None;
    }

    Some(ChartSpec {
        id: format!("bar-{}-{}", category_column.index, numeric_column.index),
        chart_type: ChartType::Bar,
        title: format!("Average {} by {}", numeric_column.name, category_column.name),
        rationale: format!(
            "{} is a grouping field and {} is numeric, so bars make the category comparison easy to scan.",
            category_column.name, numeric_column.name
        ),
        x: field_ref(category_column),
        y: field_ref(numeric_column),
        series: None,
        aggregation: Some(Aggregation::Average),
        confidence: combined_confidence(&[category_column, numeric_column]),
    })
}

fn correlation_spec(preview: &CsvPreview, profiles: &[ColumnProfile]) -> Option<ChartSpec> {
    let numeric_columns = profiles
        .iter()
        .filter(|profile| profile.inferred_type == InferredColumnType::Numeric)
        .collect::<Vec<_>>();

    let mut best: Option<(&ColumnProfile, &ColumnProfile, f64)> = None;
    for (left_index, left) in numeric_columns.iter().enumerate() {
        for right in numeric_columns.iter().skip(left_index + 1) {
            let Some(correlation) = numeric_correlation(preview, left, right) else {
                continue;
            };
            if correlation.abs() < 0.35 {
                continue;
            }
            let replace = best
                .map(|(_, _, current)| correlation.abs() > current.abs())
                .unwrap_or(true);
            if replace {
                best = Some((left, right, correlation));
            }
        }
    }

    let (x_column, y_column, correlation) = best?;
    let direction = if correlation > 0.0 { "positive" } else { "negative" };

    Some(ChartSpec {
        id: format!("scatter-{}-{}", x_column.index, y_column.index),
        chart_type: ChartType::Scatter,
        title: format!("{} vs {}", y_column.name, x_column.name),
        rationale: format!(
            "{} and {} have a {} relationship in the preview data, so a scatter plot can show whether they move together.",
            x_column.name, y_column.name, direction
        ),
        x: field_ref(x_column),
        y: field_ref(y_column),
        series: None,
        aggregation: None,
        confidence: (0.5 + correlation.abs() as f32 * 0.45).min(0.95),
    })
}

fn grouped_numeric_values(
    preview: &CsvPreview,
    category_column: &ColumnProfile,
    numeric_column: &ColumnProfile,
) -> HashMap<String, Vec<f64>> {
    let mut groups = HashMap::new();

    for row in &preview.rows {
        let Some(category) = row
            .get(category_column.index)
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

        groups
            .entry(category.to_owned())
            .or_insert_with(Vec::new)
            .push(value);
    }

    groups
}

fn numeric_correlation(
    preview: &CsvPreview,
    left: &ColumnProfile,
    right: &ColumnProfile,
) -> Option<f64> {
    let pairs = preview
        .rows
        .iter()
        .filter_map(|row| {
            let left_value = row
                .get(left.index)
                .and_then(|value| parse_number(value.as_str()))?;
            let right_value = row
                .get(right.index)
                .and_then(|value| parse_number(value.as_str()))?;

            Some((left_value, right_value))
        })
        .collect::<Vec<_>>();

    if pairs.len() < 3 {
        return None;
    }

    let count = pairs.len() as f64;
    let left_mean = pairs.iter().map(|(left, _)| left).sum::<f64>() / count;
    let right_mean = pairs.iter().map(|(_, right)| right).sum::<f64>() / count;
    let mut covariance = 0.0;
    let mut left_variance = 0.0;
    let mut right_variance = 0.0;

    for (left_value, right_value) in pairs {
        let left_delta = left_value - left_mean;
        let right_delta = right_value - right_mean;
        covariance += left_delta * right_delta;
        left_variance += left_delta.powi(2);
        right_variance += right_delta.powi(2);
    }

    if nearly_zero(left_variance) || nearly_zero(right_variance) {
        return None;
    }

    Some(covariance / (left_variance.sqrt() * right_variance.sqrt()))
}

fn field_ref(profile: &ColumnProfile) -> ChartField {
    ChartField {
        index: profile.index,
        name: profile.name.clone(),
    }
}

fn combined_confidence(profiles: &[&ColumnProfile]) -> f32 {
    if profiles.is_empty() {
        return 0.0;
    }

    let average = profiles
        .iter()
        .map(|profile| profile.confidence)
        .sum::<f32>()
        / profiles.len() as f32;

    average.min(0.95)
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

fn nearly_zero(value: f64) -> bool {
    value.abs() < f64::EPSILON
}

#[cfg(test)]
mod tests {
    use crate::{csv_parser::parse_csv_preview, profiling::profile_columns};

    use super::{select_chart_specs, ChartType};

    #[test]
    fn selects_line_bar_and_scatter_specs_from_data_shape() {
        let preview = parse_csv_preview(
            b"season,crop_type,yield_bu,input_cost\n2024,Corn,180,80\n2025,Corn,195,90\n2024,Soybean,52,30\n2025,Soybean,58,35\n",
            25,
        )
        .expect("CSV should parse");
        let profiles = profile_columns(&preview);
        let specs = select_chart_specs(&preview, &profiles);

        assert!(specs.iter().any(|spec| spec.chart_type == ChartType::Line));
        assert!(specs.iter().any(|spec| spec.chart_type == ChartType::Bar));
        assert!(specs
            .iter()
            .any(|spec| spec.chart_type == ChartType::Scatter));
    }

    #[test]
    fn skips_chart_specs_when_data_shape_is_too_thin() {
        let preview = parse_csv_preview(b"notes\nNeeds lime\nWatch drainage\n", 25)
            .expect("CSV should parse");
        let profiles = profile_columns(&preview);
        let specs = select_chart_specs(&preview, &profiles);

        assert!(specs.is_empty());
    }
}
