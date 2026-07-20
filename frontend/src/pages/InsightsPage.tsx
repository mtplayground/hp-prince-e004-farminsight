import { BarChart3, Database, FileSpreadsheet, LineChart, UploadCloud } from 'lucide-react';
import { useState } from 'react';

import type { DatasetRecord } from '../api/datasets';
import { DatasetUploadPanel } from '../components/DatasetUploadPanel';
import { MetricTile } from '../layout/AppShell';

const insightStages = [
  {
    label: 'Upload',
    description: 'CSV files enter the solo workspace and stay tied to the current user session.',
    icon: UploadCloud,
  },
  {
    label: 'Profile',
    description: 'Column types, meanings, and dataset shape appear before insight generation.',
    icon: FileSpreadsheet,
  },
  {
    label: 'Explain',
    description: 'Plain-language summaries and chart choices share one focused canvas.',
    icon: LineChart,
  },
];

export function InsightsPage() {
  const [activeDataset, setActiveDataset] = useState<DatasetRecord | null>(null);

  return (
    <div className="space-y-8">
      <section className="grid gap-6 xl:grid-cols-[1fr_320px]">
        <div>
          <p className="text-sm font-semibold uppercase tracking-wide text-harvest">
            Solo workspace
          </p>
          <h2 className="mt-2 text-3xl font-semibold tracking-normal sm:text-4xl">Insights</h2>
          <p className="mt-3 max-w-3xl text-base leading-7 text-stone-700">
            A focused path for turning one CSV dataset into schema context, plain-language findings,
            and adaptive charts.
          </p>
        </div>
        <div className="rounded-md border border-stone-200 bg-white p-5">
          <div className="flex items-center gap-3">
            <span className="flex h-10 w-10 items-center justify-center rounded-md bg-field text-white">
              <Database className="h-5 w-5" aria-hidden="true" />
            </span>
            <div>
              <p className="font-semibold">Active dataset</p>
              <p className="text-sm text-stone-600">
                {activeDataset
                  ? `${activeDataset.original_filename} · ${activeDataset.row_count ?? 0} rows`
                  : 'No dataset selected'}
              </p>
            </div>
          </div>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-3" aria-label="Insight workspace metrics">
        <MetricTile label="Datasets" value={activeDataset ? '1' : '0'} icon={Database} />
        <MetricTile label="Insight sets" value="0" icon={BarChart3} />
        <MetricTile label="Chart specs" value="0" icon={LineChart} />
      </section>

      <DatasetUploadPanel onDatasetUploaded={setActiveDataset} />

      <section className="grid gap-4 lg:grid-cols-3" aria-label="Insight workflow">
        {insightStages.map(({ label, description, icon: Icon }) => (
          <article key={label} className="rounded-md border border-stone-200 bg-white p-5">
            <Icon className="h-5 w-5 text-field" aria-hidden="true" />
            <h3 className="mt-4 text-base font-semibold">{label}</h3>
            <p className="mt-2 text-sm leading-6 text-stone-600">{description}</p>
          </article>
        ))}
      </section>

      <section className="rounded-md border border-dashed border-stone-300 bg-white p-6">
        <div className="max-w-2xl">
          <h3 className="text-lg font-semibold">Insight canvas</h3>
          <p className="mt-2 text-sm leading-6 text-stone-600">
            {activeDataset
              ? `${activeDataset.original_filename} is ready for schema profiling and insight generation.`
              : 'No dataset selected. This primary route keeps previews, summaries, charts, and drill-down controls in one focused workspace.'}
          </p>
        </div>
      </section>
    </div>
  );
}
