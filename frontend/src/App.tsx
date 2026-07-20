import { Activity, BarChart3, Database, Layers3, UploadCloud, Users } from 'lucide-react';
import { useEffect, useState } from 'react';

type HealthState =
  | { status: 'checking' }
  | { status: 'ready'; service: string; version: string }
  | { status: 'unavailable'; message: string };

const navItems = [
  { label: 'Datasets', icon: Database },
  { label: 'Insights', icon: BarChart3 },
  { label: 'Team', icon: Users },
];

const workflowItems = [
  {
    title: 'Upload workspace',
    description: 'CSV intake and preview will live here once dataset ingestion is added.',
    icon: UploadCloud,
  },
  {
    title: 'Schema intelligence',
    description:
      'Detected column types and summary statistics will be surfaced from the backend API.',
    icon: Layers3,
  },
  {
    title: 'Insight gallery',
    description: 'Plain-language summaries and adaptive chart specs will render in this area.',
    icon: BarChart3,
  },
];

function App() {
  const [health, setHealth] = useState<HealthState>({ status: 'checking' });

  useEffect(() => {
    const controller = new AbortController();

    fetch('/api/health', { signal: controller.signal })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`API returned ${response.status}`);
        }
        return response.json() as Promise<{ service: string; version: string }>;
      })
      .then((body) => {
        setHealth({ status: 'ready', service: body.service, version: body.version });
      })
      .catch((error: unknown) => {
        if (controller.signal.aborted) {
          return;
        }
        const message = error instanceof Error ? error.message : 'Unable to reach the API';
        setHealth({ status: 'unavailable', message });
      });

    return () => controller.abort();
  }, []);

  return (
    <div className="min-h-screen bg-stone-50 text-ink">
      <header className="border-b border-stone-200 bg-white">
        <div className="mx-auto flex max-w-7xl items-center justify-between px-5 py-4">
          <div>
            <p className="text-xs font-semibold uppercase tracking-wide text-field">
              CSV analytics
            </p>
            <h1 className="text-2xl font-semibold">Farminsight</h1>
          </div>
          <nav className="hidden items-center gap-2 md:flex" aria-label="Primary">
            {navItems.map(({ label, icon: Icon }) => (
              <button
                key={label}
                className="inline-flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium text-stone-700 hover:bg-stone-100"
                type="button"
              >
                <Icon className="h-4 w-4" aria-hidden="true" />
                {label}
              </button>
            ))}
          </nav>
        </div>
      </header>

      <main className="mx-auto grid max-w-7xl gap-8 px-5 py-8 lg:grid-cols-[280px_1fr]">
        <aside className="border-b border-stone-200 pb-6 lg:border-b-0 lg:border-r lg:pr-6">
          <div className="flex items-center gap-3">
            <span className="flex h-10 w-10 items-center justify-center rounded-md bg-field text-white">
              <Activity className="h-5 w-5" aria-hidden="true" />
            </span>
            <div>
              <p className="text-sm font-semibold">Runtime status</p>
              <p className="text-sm text-stone-600">SPA served by Axum</p>
            </div>
          </div>

          <div className="mt-5 rounded-md border border-stone-200 bg-white p-4">
            {health.status === 'checking' && <StatusLabel tone="neutral" text="Checking API" />}
            {health.status === 'ready' && (
              <div className="space-y-2">
                <StatusLabel tone="success" text="API ready" />
                <p className="break-words text-sm text-stone-600">
                  {health.service} v{health.version}
                </p>
              </div>
            )}
            {health.status === 'unavailable' && (
              <div className="space-y-2">
                <StatusLabel tone="warning" text="API unavailable" />
                <p className="break-words text-sm text-stone-600">{health.message}</p>
              </div>
            )}
          </div>
        </aside>

        <section aria-labelledby="workspace-heading">
          <div className="max-w-3xl">
            <p className="text-sm font-semibold uppercase tracking-wide text-harvest">Workspace</p>
            <h2 id="workspace-heading" className="mt-2 text-3xl font-semibold">
              Analyze agricultural CSV data from upload to insight.
            </h2>
            <p className="mt-3 text-base leading-7 text-stone-700">
              The repository now has a React SPA, Tailwind styling, and an Axum host ready for the
              feature-specific issues that will add authentication, datasets, teams, and analytics.
            </p>
          </div>

          <div className="mt-8 grid gap-4 md:grid-cols-3">
            {workflowItems.map(({ title, description, icon: Icon }) => (
              <article key={title} className="rounded-md border border-stone-200 bg-white p-5">
                <Icon className="h-5 w-5 text-field" aria-hidden="true" />
                <h3 className="mt-4 text-base font-semibold">{title}</h3>
                <p className="mt-2 text-sm leading-6 text-stone-600">{description}</p>
              </article>
            ))}
          </div>
        </section>
      </main>
    </div>
  );
}

function StatusLabel({ text, tone }: { text: string; tone: 'neutral' | 'success' | 'warning' }) {
  const toneClass = {
    neutral: 'bg-stone-100 text-stone-700',
    success: 'bg-green-50 text-green-700',
    warning: 'bg-amber-50 text-amber-700',
  }[tone];

  return (
    <span className={`inline-flex rounded-md px-2.5 py-1 text-xs font-semibold ${toneClass}`}>
      {text}
    </span>
  );
}

export default App;
