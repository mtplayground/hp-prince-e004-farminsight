import { BarChart3, LineChart, ScatterChart as ScatterIcon } from 'lucide-react';
import {
  Bar,
  BarChart,
  CartesianGrid,
  Line,
  LineChart as RechartsLineChart,
  ResponsiveContainer,
  Scatter,
  ScatterChart,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';

import type { CsvPreview, DatasetChartSpec, DatasetRecord } from '../api/datasets';

type AdaptiveChartGalleryProps = {
  dataset: DatasetRecord | null;
  preview: CsvPreview | null;
  chartSpecs: DatasetChartSpec[];
};

type ChartDatum = {
  label: string;
  value: number;
  xValue: number;
};

const chartColors = {
  line: '#2f6f4e',
  bar: '#8a6116',
  scatter: '#365f8c',
};

export function AdaptiveChartGallery({ dataset, preview, chartSpecs }: AdaptiveChartGalleryProps) {
  if (!dataset) {
    return (
      <section className="rounded-md border border-stone-200 bg-white p-6">
        <div className="flex items-start gap-3">
          <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-field text-white">
            <BarChart3 className="h-5 w-5" aria-hidden="true" />
          </span>
          <div>
            <h3 className="text-lg font-semibold">Adaptive chart gallery</h3>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-stone-600">No dataset selected.</p>
          </div>
        </div>
      </section>
    );
  }

  if (!preview || chartSpecs.length === 0) {
    return (
      <section className="rounded-md border border-stone-200 bg-white p-6">
        <div className="flex items-start gap-3">
          <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-field text-white">
            <BarChart3 className="h-5 w-5" aria-hidden="true" />
          </span>
          <div>
            <h3 className="text-lg font-semibold">Adaptive chart gallery</h3>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-stone-600">
              {dataset.original_filename} has no chart specs available yet.
            </p>
          </div>
        </div>
      </section>
    );
  }

  return (
    <section className="space-y-4" aria-label="Adaptive chart gallery">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-sm font-semibold uppercase tracking-wide text-harvest">
            Adaptive chart gallery
          </p>
          <h3 className="mt-2 text-2xl font-semibold tracking-normal text-stone-950">
            {dataset.original_filename}
          </h3>
        </div>
        <p className="text-sm text-stone-600">
          {chartSpecs.length.toLocaleString()} chart {chartSpecs.length === 1 ? 'spec' : 'specs'}
        </p>
      </div>

      <div className="grid gap-4 xl:grid-cols-2">
        {chartSpecs.map((spec) => (
          <ChartCard key={spec.id} preview={preview} spec={spec} />
        ))}
      </div>
    </section>
  );
}

function ChartCard({ preview, spec }: { preview: CsvPreview; spec: DatasetChartSpec }) {
  const data = chartData(preview, spec);
  const Icon =
    spec.chart_type === 'scatter'
      ? ScatterIcon
      : spec.chart_type === 'line'
        ? LineChart
        : BarChart3;

  return (
    <article className="rounded-md border border-stone-200 bg-white p-5">
      <div className="flex items-start justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <Icon className="h-4 w-4 text-field" aria-hidden="true" />
            <p className="text-xs font-semibold uppercase tracking-wide text-harvest">
              {chartTypeLabel(spec.chart_type)}
            </p>
          </div>
          <h4 className="mt-2 text-base font-semibold text-stone-950">{spec.title}</h4>
          <p className="mt-2 text-sm leading-6 text-stone-600">{spec.rationale}</p>
        </div>
        <span className="rounded-md border border-stone-200 bg-stone-50 px-2 py-1 text-xs font-semibold text-stone-600">
          {Math.round(spec.confidence * 100)}%
        </span>
      </div>

      <div className="mt-5 h-72 rounded-md border border-stone-200 bg-stone-50 p-3">
        {data.length > 0 ? renderChart(spec, data) : <EmptyChartState />}
      </div>

      <div className="mt-3 flex flex-wrap gap-2 text-xs text-stone-600">
        <span className="rounded-md bg-stone-100 px-2 py-1">X: {spec.x.name}</span>
        <span className="rounded-md bg-stone-100 px-2 py-1">Y: {spec.y.name}</span>
        {spec.series && (
          <span className="rounded-md bg-stone-100 px-2 py-1">Series: {spec.series.name}</span>
        )}
      </div>
    </article>
  );
}

function renderChart(spec: DatasetChartSpec, data: ChartDatum[]) {
  if (spec.chart_type === 'line') {
    return (
      <ResponsiveContainer width="100%" height="100%">
        <RechartsLineChart data={data} margin={{ top: 10, right: 16, bottom: 10, left: 0 }}>
          <CartesianGrid stroke="#e7e5e4" strokeDasharray="4 4" />
          <XAxis dataKey="label" tick={{ fontSize: 12 }} minTickGap={20} />
          <YAxis tick={{ fontSize: 12 }} width={44} />
          <Tooltip formatter={(value) => formatTooltipValue(value)} />
          <Line
            type="monotone"
            dataKey="value"
            stroke={chartColors.line}
            strokeWidth={2}
            dot={{ r: 3 }}
            activeDot={{ r: 5 }}
          />
        </RechartsLineChart>
      </ResponsiveContainer>
    );
  }

  if (spec.chart_type === 'scatter') {
    return (
      <ResponsiveContainer width="100%" height="100%">
        <ScatterChart margin={{ top: 10, right: 16, bottom: 10, left: 0 }}>
          <CartesianGrid stroke="#e7e5e4" strokeDasharray="4 4" />
          <XAxis dataKey="xValue" name={spec.x.name} tick={{ fontSize: 12 }} type="number" />
          <YAxis dataKey="value" name={spec.y.name} tick={{ fontSize: 12 }} width={44} />
          <Tooltip
            cursor={{ strokeDasharray: '4 4' }}
            formatter={(value) => formatTooltipValue(value)}
          />
          <Scatter data={data} fill={chartColors.scatter} />
        </ScatterChart>
      </ResponsiveContainer>
    );
  }

  return (
    <ResponsiveContainer width="100%" height="100%">
      <BarChart data={data} margin={{ top: 10, right: 16, bottom: 10, left: 0 }}>
        <CartesianGrid stroke="#e7e5e4" strokeDasharray="4 4" />
        <XAxis dataKey="label" tick={{ fontSize: 12 }} minTickGap={16} />
        <YAxis tick={{ fontSize: 12 }} width={44} />
        <Tooltip formatter={(value) => formatTooltipValue(value)} />
        <Bar dataKey="value" fill={chartColors.bar} radius={[3, 3, 0, 0]} />
      </BarChart>
    </ResponsiveContainer>
  );
}

function chartData(preview: CsvPreview, spec: DatasetChartSpec) {
  if (spec.chart_type === 'scatter') {
    return scatterData(preview, spec);
  }

  return groupedAverageData(preview, spec);
}

function groupedAverageData(preview: CsvPreview, spec: DatasetChartSpec) {
  const groups = new Map<string, { sortValue: number; sum: number; count: number }>();

  for (const row of preview.rows) {
    const label = row[spec.x.index]?.trim();
    const value = parseNumber(row[spec.y.index]);
    if (!label || value === null) {
      continue;
    }

    const current = groups.get(label) ?? { sortValue: sortValue(label), sum: 0, count: 0 };
    current.sum += value;
    current.count += 1;
    groups.set(label, current);
  }

  return Array.from(groups, ([label, group]) => ({
    label,
    value: group.sum / group.count,
    xValue: group.sortValue,
  }))
    .sort((left, right) => left.xValue - right.xValue || left.label.localeCompare(right.label))
    .slice(0, 24);
}

function scatterData(preview: CsvPreview, spec: DatasetChartSpec) {
  return preview.rows
    .map((row) => {
      const xValue = parseNumber(row[spec.x.index]);
      const value = parseNumber(row[spec.y.index]);
      if (xValue === null || value === null) {
        return null;
      }

      return {
        label: row[spec.x.index] || `${xValue}`,
        xValue,
        value,
      };
    })
    .filter((datum): datum is ChartDatum => datum !== null)
    .slice(0, 200);
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

function sortValue(value: string) {
  const numeric = parseNumber(value);
  if (numeric !== null) {
    return numeric;
  }

  const timestamp = Date.parse(value);
  if (Number.isFinite(timestamp)) {
    return timestamp;
  }

  return Number.MAX_SAFE_INTEGER;
}

function chartTypeLabel(chartType: string) {
  return chartType.replaceAll('_', ' ');
}

function formatTooltipValue(value: unknown): string | number {
  if (typeof value === 'number') {
    return value.toLocaleString(undefined, { maximumFractionDigits: 2 });
  }

  if (typeof value === 'string') {
    return value;
  }

  return '';
}

function EmptyChartState() {
  return (
    <div className="flex h-full items-center justify-center px-4 text-center text-sm text-stone-500">
      Not enough numeric rows for this chart.
    </div>
  );
}
