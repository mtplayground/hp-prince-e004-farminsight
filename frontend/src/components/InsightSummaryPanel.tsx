import { AlertTriangle, BarChart3, CheckCircle2, Loader2, Sparkles } from 'lucide-react';
import { useEffect, useState } from 'react';

import {
  type DatasetChartSpec,
  type DatasetInsight,
  type DatasetInsightsResponse,
  type DatasetRecord,
  fetchDatasetInsights,
} from '../api/datasets';

type InsightLoadState = 'idle' | 'loading' | 'loaded' | 'error';

type InsightSummaryPanelProps = {
  dataset: DatasetRecord | null;
  onInsightsLoaded?: (insights: DatasetInsightsResponse) => void;
};

type LoadedInsights = {
  datasetId: string;
  insights: DatasetInsight[];
  chartSpecs: DatasetChartSpec[];
  status: Exclude<InsightLoadState, 'idle'>;
  error: string | null;
};

export function InsightSummaryPanel({ dataset, onInsightsLoaded }: InsightSummaryPanelProps) {
  const [loadedInsights, setLoadedInsights] = useState<LoadedInsights | null>(null);

  useEffect(() => {
    if (!dataset) {
      return;
    }

    const controller = new AbortController();
    fetchDatasetInsights(dataset.id, controller.signal)
      .then((response) => {
        setLoadedInsights({
          datasetId: dataset.id,
          insights: response.insights,
          chartSpecs: response.chart_specs,
          status: 'loaded',
          error: null,
        });
        onInsightsLoaded?.(response);
      })
      .catch((cause: unknown) => {
        if (controller.signal.aborted) {
          return;
        }
        setLoadedInsights({
          datasetId: dataset.id,
          insights: dataset.cached_insights,
          chartSpecs: dataset.cached_chart_specs,
          status: dataset.cached_insights.length > 0 ? 'loaded' : 'error',
          error: cause instanceof Error ? cause.message : 'Dataset insights fetch failed',
        });
      });

    return () => controller.abort();
  }, [dataset, onInsightsLoaded]);

  const currentLoadedInsights =
    dataset && loadedInsights?.datasetId === dataset.id ? loadedInsights : null;
  const insights = currentLoadedInsights?.insights ?? dataset?.cached_insights ?? [];
  const chartSpecs = currentLoadedInsights?.chartSpecs ?? dataset?.cached_chart_specs ?? [];
  const status: InsightLoadState = !dataset ? 'idle' : (currentLoadedInsights?.status ?? 'loading');
  const error = currentLoadedInsights?.error ?? null;
  const parserWarnings = dataset ? parserWarningList(dataset.stats) : [];
  const topInsight = insights[0] ?? null;
  const supportingInsights = insights.slice(1, 4);

  if (!dataset) {
    return (
      <section className="rounded-md border border-stone-200 bg-white p-6">
        <div className="flex items-start gap-3">
          <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-field text-white">
            <Sparkles className="h-5 w-5" aria-hidden="true" />
          </span>
          <div>
            <p className="text-sm font-semibold uppercase tracking-wide text-harvest">
              Plain-language summary
            </p>
            <h3 className="mt-2 text-xl font-semibold">No dataset selected</h3>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-stone-600">
              Commit a CSV dataset to see the first readable takeaway here before reviewing chart
              recommendations.
            </p>
          </div>
        </div>
      </section>
    );
  }

  return (
    <section className="rounded-md border border-field/20 bg-white p-6 shadow-sm shadow-green-900/5">
      <div className="flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
        <div className="max-w-3xl">
          <p className="text-sm font-semibold uppercase tracking-wide text-harvest">
            Plain-language summary
          </p>
          <h3 className="mt-2 text-2xl font-semibold tracking-normal text-stone-950">
            {topInsight?.title ?? 'Summary is being prepared'}
          </h3>
          <p className="mt-3 text-base leading-7 text-stone-700">
            {topInsight?.summary ??
              `${dataset.original_filename} has been committed and is being checked for readable findings.`}
          </p>
        </div>

        <div className="grid min-w-52 grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-1">
          <SummaryStat label="Findings" value={insights.length.toLocaleString()} />
          <SummaryStat label="Charts" value={chartSpecs.length.toLocaleString()} />
          <SummaryStat
            label="Confidence"
            value={topInsight ? `${Math.round(topInsight.confidence * 100)}%` : 'Pending'}
          />
        </div>
      </div>

      {status === 'loading' && (
        <div className="mt-5 flex items-center gap-2 rounded-md border border-stone-200 bg-stone-50 p-3 text-sm text-stone-600">
          <Loader2 className="h-4 w-4 animate-spin" aria-hidden="true" />
          Refreshing summary
        </div>
      )}

      {error && (
        <div className="mt-5 flex gap-2 rounded-md border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800">
          <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
          <p>{error}</p>
        </div>
      )}

      {parserWarnings.length > 0 && (
        <div className="mt-5 rounded-md border border-amber-200 bg-amber-50 p-4">
          <div className="flex items-center gap-2 text-sm font-semibold text-amber-900">
            <AlertTriangle className="h-4 w-4" aria-hidden="true" />
            Parser recovery notes
          </div>
          <ul className="mt-2 space-y-1 text-sm leading-6 text-amber-800">
            {parserWarnings.map((warning) => (
              <li key={warning}>{warning}</li>
            ))}
          </ul>
        </div>
      )}

      {topInsight?.evidence && topInsight.evidence.length > 0 && (
        <div className="mt-5 rounded-md border border-stone-200 bg-stone-50 p-4">
          <p className="text-sm font-semibold text-stone-800">Evidence</p>
          <ul className="mt-2 space-y-2 text-sm leading-6 text-stone-600">
            {topInsight.evidence.map((item) => (
              <li key={item} className="flex gap-2">
                <CheckCircle2 className="mt-1 h-4 w-4 shrink-0 text-field" aria-hidden="true" />
                <span>{item}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {supportingInsights.length > 0 && (
        <div className="mt-5 grid gap-3 lg:grid-cols-3">
          {supportingInsights.map((insight) => (
            <article
              key={`${insight.kind}-${insight.title}`}
              className="rounded-md border border-stone-200 p-4"
            >
              <p className="text-xs font-semibold uppercase tracking-wide text-harvest">
                {kindLabel(insight.kind)}
              </p>
              <h4 className="mt-2 text-sm font-semibold text-stone-950">{insight.title}</h4>
              <p className="mt-2 text-sm leading-6 text-stone-600">{insight.summary}</p>
            </article>
          ))}
        </div>
      )}

      {chartSpecs.length > 0 && (
        <div className="mt-6 border-t border-stone-200 pt-5">
          <div className="flex items-center gap-2">
            <BarChart3 className="h-4 w-4 text-field" aria-hidden="true" />
            <h4 className="text-sm font-semibold text-stone-950">Chart recommendations</h4>
          </div>
          <div className="mt-3 grid gap-3 md:grid-cols-2">
            {chartSpecs.slice(0, 2).map((spec) => (
              <article key={spec.id} className="rounded-md border border-stone-200 bg-stone-50 p-4">
                <p className="text-sm font-semibold text-stone-900">{spec.title}</p>
                <p className="mt-2 text-sm leading-6 text-stone-600">{spec.rationale}</p>
              </article>
            ))}
          </div>
        </div>
      )}
    </section>
  );
}

function SummaryStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-stone-200 bg-stone-50 p-3">
      <p className="text-xs font-medium text-stone-500">{label}</p>
      <p className="mt-1 break-words text-lg font-semibold text-stone-950">{value}</p>
    </div>
  );
}

function kindLabel(kind: string) {
  return kind.replaceAll('_', ' ');
}

function parserWarningList(stats: Record<string, unknown>) {
  const warnings = stats.parser_warnings;
  if (!Array.isArray(warnings)) {
    return [];
  }

  return warnings.filter((warning): warning is string => typeof warning === 'string').slice(0, 3);
}
