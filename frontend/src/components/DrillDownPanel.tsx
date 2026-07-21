import { Filter, Table2 } from 'lucide-react';
import { useMemo, useState } from 'react';

import type { CsvPreview, DatasetChartSpec, DatasetRecord } from '../api/datasets';

type DrillDownPanelProps = {
  dataset: DatasetRecord | null;
  preview: CsvPreview | null;
  chartSpecs: DatasetChartSpec[];
};

type MetricOption = {
  index: number;
  name: string;
};

type FieldOption = {
  index: number;
  name: string;
  values: string[];
};

type SeasonOption = {
  columnIndex: number;
  columnName: string;
  year: string;
};

const allValue = '__all__';

export function DrillDownPanel({ dataset, preview, chartSpecs }: DrillDownPanelProps) {
  const [fieldColumnIndex, setFieldColumnIndex] = useState<string>(allValue);
  const [fieldValue, setFieldValue] = useState<string>(allValue);
  const [seasonValue, setSeasonValue] = useState<string>(allValue);
  const [metricIndex, setMetricIndex] = useState<string>(allValue);

  const fieldOptions = useMemo(
    () => (preview ? fieldOptionsFor(preview, chartSpecs) : []),
    [preview, chartSpecs],
  );
  const seasonOptions = useMemo(
    () => (preview ? seasonOptionsFor(preview, chartSpecs) : []),
    [preview, chartSpecs],
  );
  const metricOptions = useMemo(
    () => (preview ? metricOptionsFor(preview, chartSpecs) : []),
    [preview, chartSpecs],
  );

  if (!dataset || !preview) {
    return (
      <section className="rounded-md border border-stone-200 bg-white p-6">
        <div className="flex items-start gap-3">
          <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-field text-white">
            <Filter className="h-5 w-5" aria-hidden="true" />
          </span>
          <div>
            <h3 className="text-lg font-semibold">Drill-down</h3>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-stone-600">No dataset selected.</p>
          </div>
        </div>
      </section>
    );
  }

  const activeFieldColumn = fieldOptions.find((option) => `${option.index}` === fieldColumnIndex);
  const activeMetric =
    metricOptions.find((option) => `${option.index}` === metricIndex) ??
    metricOptions.find((option) => chartSpecs.some((spec) => spec.y.index === option.index)) ??
    metricOptions[0] ??
    null;
  const activeSeason = seasonOptions.find((option) => seasonKey(option) === seasonValue) ?? null;
  const activeFieldValue =
    activeFieldColumn?.values.includes(fieldValue) === true ? fieldValue : allValue;

  const filteredRows = preview.rows.filter((row) => {
    if (activeFieldColumn && activeFieldValue !== allValue) {
      if ((row[activeFieldColumn.index] ?? '').trim() !== activeFieldValue) {
        return false;
      }
    }

    if (activeSeason) {
      const rowYear = yearFromValue(row[activeSeason.columnIndex]);
      if (rowYear !== activeSeason.year) {
        return false;
      }
    }

    return true;
  });
  const metricStats = activeMetric ? statsForMetric(filteredRows, activeMetric.index) : null;
  const detailRows = filteredRows.slice(0, 8);

  return (
    <section className="rounded-md border border-stone-200 bg-white p-6">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-sm font-semibold uppercase tracking-wide text-harvest">Drill-down</p>
          <h3 className="mt-2 text-2xl font-semibold tracking-normal text-stone-950">
            Field, season, and metric detail
          </h3>
        </div>
        <p className="text-sm text-stone-600">
          {filteredRows.length.toLocaleString()} of {preview.row_count.toLocaleString()} rows
        </p>
      </div>

      <div className="mt-5 grid gap-3 lg:grid-cols-4">
        <label className="block">
          <span className="text-xs font-semibold uppercase tracking-wide text-stone-500">
            Field column
          </span>
          <select
            value={activeFieldColumn ? `${activeFieldColumn.index}` : allValue}
            onChange={(event) => {
              setFieldColumnIndex(event.currentTarget.value);
              setFieldValue(allValue);
            }}
            className="mt-2 h-10 w-full rounded-md border border-stone-300 bg-white px-3 text-sm text-stone-900"
          >
            <option value={allValue}>All fields</option>
            {fieldOptions.map((option) => (
              <option key={option.index} value={option.index}>
                {option.name}
              </option>
            ))}
          </select>
        </label>

        <label className="block">
          <span className="text-xs font-semibold uppercase tracking-wide text-stone-500">
            Field value
          </span>
          <select
            value={activeFieldValue}
            onChange={(event) => setFieldValue(event.currentTarget.value)}
            disabled={!activeFieldColumn}
            className="mt-2 h-10 w-full rounded-md border border-stone-300 bg-white px-3 text-sm text-stone-900 disabled:bg-stone-100 disabled:text-stone-500"
          >
            <option value={allValue}>All values</option>
            {activeFieldColumn?.values.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </label>

        <label className="block">
          <span className="text-xs font-semibold uppercase tracking-wide text-stone-500">
            Season
          </span>
          <select
            value={activeSeason ? seasonKey(activeSeason) : allValue}
            onChange={(event) => setSeasonValue(event.currentTarget.value)}
            className="mt-2 h-10 w-full rounded-md border border-stone-300 bg-white px-3 text-sm text-stone-900"
          >
            <option value={allValue}>All seasons</option>
            {seasonOptions.map((option) => (
              <option key={seasonKey(option)} value={seasonKey(option)}>
                {option.year} · {option.columnName}
              </option>
            ))}
          </select>
        </label>

        <label className="block">
          <span className="text-xs font-semibold uppercase tracking-wide text-stone-500">
            Metric
          </span>
          <select
            value={activeMetric ? `${activeMetric.index}` : allValue}
            onChange={(event) => setMetricIndex(event.currentTarget.value)}
            className="mt-2 h-10 w-full rounded-md border border-stone-300 bg-white px-3 text-sm text-stone-900"
          >
            {metricOptions.length === 0 && <option value={allValue}>No numeric metric</option>}
            {metricOptions.map((option) => (
              <option key={option.index} value={option.index}>
                {option.name}
              </option>
            ))}
          </select>
        </label>
      </div>

      <div className="mt-5 grid gap-3 md:grid-cols-4">
        <DrillMetric label="Rows" value={filteredRows.length.toLocaleString()} />
        <DrillMetric
          label="Average"
          value={metricStats ? formatNumber(metricStats.average) : 'None'}
        />
        <DrillMetric label="Low" value={metricStats ? formatNumber(metricStats.min) : 'None'} />
        <DrillMetric label="High" value={metricStats ? formatNumber(metricStats.max) : 'None'} />
      </div>

      <div className="mt-5 overflow-hidden rounded-md border border-stone-200">
        <div className="flex items-center gap-2 border-b border-stone-200 bg-stone-50 px-3 py-2">
          <Table2 className="h-4 w-4 text-field" aria-hidden="true" />
          <p className="text-sm font-semibold text-stone-800">Filtered rows</p>
        </div>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-stone-200 text-left text-sm">
            <thead className="bg-white">
              <tr>
                {preview.columns.map((column) => (
                  <th
                    key={column}
                    scope="col"
                    className="whitespace-nowrap px-3 py-2 font-semibold text-stone-700"
                  >
                    {column}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-100 bg-white">
              {detailRows.map((row, rowIndex) => (
                <tr key={`${rowIndex}-${row.join('|')}`}>
                  {preview.columns.map((column, columnIndex) => (
                    <td
                      key={`${column}-${columnIndex}`}
                      className="max-w-56 truncate px-3 py-2 text-stone-700"
                    >
                      {row[columnIndex] || <span className="text-stone-400">blank</span>}
                    </td>
                  ))}
                </tr>
              ))}
              {detailRows.length === 0 && (
                <tr>
                  <td
                    className="px-3 py-6 text-center text-sm text-stone-500"
                    colSpan={preview.columns.length}
                  >
                    No rows match the current drill-down.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}

function DrillMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-stone-200 bg-stone-50 p-3">
      <p className="text-xs font-medium text-stone-500">{label}</p>
      <p className="mt-1 break-words text-lg font-semibold text-stone-950">{value}</p>
    </div>
  );
}

function fieldOptionsFor(preview: CsvPreview, chartSpecs: DatasetChartSpec[]) {
  const preferredIndexes = new Set(
    chartSpecs
      .filter((spec) => spec.chart_type === 'bar' || spec.series)
      .flatMap((spec) => [spec.x.index, spec.series?.index])
      .filter((index): index is number => typeof index === 'number'),
  );

  return preview.columns
    .map((name, index): FieldOption | null => {
      const values = uniqueValues(preview, index);
      const numericValues = preview.rows.filter((row) => parseNumber(row[index]) !== null).length;
      const mostlyNumeric = numericValues > preview.rows.length * 0.65;
      if (values.length < 1 || values.length > 50 || mostlyNumeric) {
        return null;
      }

      return { index, name, values };
    })
    .filter((option): option is FieldOption => option !== null)
    .sort(
      (left, right) =>
        Number(preferredIndexes.has(right.index)) - Number(preferredIndexes.has(left.index)),
    );
}

function seasonOptionsFor(preview: CsvPreview, chartSpecs: DatasetChartSpec[]) {
  const preferredIndexes = new Set(
    chartSpecs.filter((spec) => spec.chart_type === 'line').map((spec) => spec.x.index),
  );
  const options: SeasonOption[] = [];

  preview.columns.forEach((columnName, columnIndex) => {
    const years = Array.from(
      new Set(preview.rows.map((row) => yearFromValue(row[columnIndex])).filter(Boolean)),
    ) as string[];
    if (years.length === 0 || years.length > 25) {
      return;
    }

    years.sort();
    options.push(...years.map((year) => ({ columnIndex, columnName, year })));
  });

  return options.sort((left, right) => {
    const preferredDelta =
      Number(preferredIndexes.has(right.columnIndex)) -
      Number(preferredIndexes.has(left.columnIndex));
    return (
      preferredDelta ||
      left.columnName.localeCompare(right.columnName) ||
      left.year.localeCompare(right.year)
    );
  });
}

function metricOptionsFor(preview: CsvPreview, chartSpecs: DatasetChartSpec[]) {
  const preferredIndexes = new Set(chartSpecs.map((spec) => spec.y.index));

  return preview.columns
    .map((name, index): MetricOption | null => {
      const numericValues = preview.rows.filter((row) => parseNumber(row[index]) !== null).length;
      if (numericValues === 0 || numericValues < preview.rows.length * 0.35) {
        return null;
      }

      return { index, name };
    })
    .filter((option): option is MetricOption => option !== null)
    .sort(
      (left, right) =>
        Number(preferredIndexes.has(right.index)) - Number(preferredIndexes.has(left.index)),
    );
}

function uniqueValues(preview: CsvPreview, columnIndex: number) {
  return Array.from(
    new Set(
      preview.rows
        .map((row) => row[columnIndex]?.trim())
        .filter((value): value is string => Boolean(value)),
    ),
  )
    .sort((left, right) => left.localeCompare(right))
    .slice(0, 50);
}

function statsForMetric(rows: string[][], metricIndex: number) {
  const values = rows
    .map((row) => parseNumber(row[metricIndex]))
    .filter((value): value is number => value !== null);

  if (values.length === 0) {
    return null;
  }

  const sum = values.reduce((total, value) => total + value, 0);
  return {
    average: sum / values.length,
    min: Math.min(...values),
    max: Math.max(...values),
  };
}

function parseNumber(value: string | undefined) {
  if (!value) {
    return null;
  }

  const trimmed = value.trim();
  const parenthesized = trimmed.startsWith('(') && trimmed.endsWith(')');
  const cleaned = trimmed
    .replaceAll(',', '')
    .replace(/^\$/, '')
    .replace(/%$/, '')
    .replace(/[()]/g, '');
  const parsed = Number.parseFloat(cleaned);

  if (!Number.isFinite(parsed)) {
    return null;
  }

  return parenthesized ? -parsed : parsed;
}

function yearFromValue(value: string | undefined) {
  if (!value) {
    return null;
  }

  const trimmed = value.trim();
  if (/^\d{4}$/.test(trimmed)) {
    return trimmed;
  }

  const date = Date.parse(trimmed);
  if (Number.isFinite(date)) {
    return `${new Date(date).getUTCFullYear()}`;
  }

  return null;
}

function seasonKey(option: SeasonOption) {
  return `${option.columnIndex}:${option.year}`;
}

function formatNumber(value: number) {
  return value.toLocaleString(undefined, { maximumFractionDigits: 2 });
}
