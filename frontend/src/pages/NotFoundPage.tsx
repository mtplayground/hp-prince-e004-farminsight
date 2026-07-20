import { Link } from 'react-router-dom';

export function NotFoundPage() {
  return (
    <section className="rounded-md border border-stone-200 bg-white p-6">
      <p className="text-sm font-semibold uppercase tracking-wide text-harvest">Route not found</p>
      <h2 className="mt-2 text-2xl font-semibold">This workspace page does not exist.</h2>
      <p className="mt-3 max-w-2xl text-sm leading-6 text-stone-600">
        Return to the Insights workspace or use the primary navigation to switch sections.
      </p>
      <Link
        to="/insights"
        className="mt-5 inline-flex min-h-10 items-center rounded-md bg-field px-4 text-sm font-semibold text-white hover:bg-green-700"
      >
        Open Insights
      </Link>
    </section>
  );
}
