import { ArrowRight, BarChart3, ShieldCheck, type LucideIcon } from 'lucide-react';
import { Link } from 'react-router-dom';

export function AuthPanel({
  eyebrow,
  title,
  description,
  primaryLabel,
  primaryIcon: PrimaryIcon,
  primaryHref,
  onPrimaryClick,
  secondaryLabel,
  secondaryTo,
}: {
  eyebrow: string;
  title: string;
  description: string;
  primaryLabel: string;
  primaryIcon: LucideIcon;
  primaryHref: string;
  onPrimaryClick: () => void;
  secondaryLabel: string;
  secondaryTo: string;
}) {
  return (
    <div className="mx-auto grid max-w-5xl gap-6 lg:grid-cols-[1fr_360px]">
      <section className="rounded-md border border-stone-200 bg-white p-6 sm:p-8">
        <p className="text-sm font-semibold uppercase tracking-wide text-harvest">{eyebrow}</p>
        <h2 className="mt-2 text-3xl font-semibold tracking-normal sm:text-4xl">{title}</h2>
        <p className="mt-3 max-w-2xl text-base leading-7 text-stone-700">{description}</p>

        <div className="mt-8 flex flex-col gap-3 sm:flex-row">
          <a
            href={primaryHref}
            onClick={(event) => {
              event.preventDefault();
              onPrimaryClick();
            }}
            className="inline-flex min-h-11 items-center justify-center gap-2 rounded-md bg-field px-5 text-sm font-semibold text-white hover:bg-green-700"
          >
            <PrimaryIcon className="h-4 w-4" aria-hidden="true" />
            {primaryLabel}
            <ArrowRight className="h-4 w-4" aria-hidden="true" />
          </a>
          <Link
            to={secondaryTo}
            className="inline-flex min-h-11 items-center justify-center rounded-md border border-stone-300 bg-white px-5 text-sm font-semibold text-stone-800 hover:bg-stone-100"
          >
            {secondaryLabel}
          </Link>
        </div>
      </section>

      <aside className="rounded-md border border-stone-200 bg-white p-6">
        <div className="flex h-11 w-11 items-center justify-center rounded-md bg-field text-white">
          <ShieldCheck className="h-5 w-5" aria-hidden="true" />
        </div>
        <h3 className="mt-5 text-base font-semibold">Ready for CSV work</h3>
        <p className="mt-2 text-sm leading-6 text-stone-600">
          Keep datasets, summaries, chart galleries, and team access available from one workspace.
        </p>
        <div className="mt-5 border-t border-stone-200 pt-5">
          <BarChart3 className="h-5 w-5 text-field" aria-hidden="true" />
          <p className="mt-3 text-sm font-semibold">Return path</p>
          <p className="mt-1 break-words text-sm text-stone-600">Insights workspace</p>
        </div>
      </aside>
    </div>
  );
}
