import { Database, ShieldCheck, UserPlus, Users } from 'lucide-react';

import { MetricTile } from '../layout/AppShell';

const teamSections = [
  {
    title: 'Shared datasets',
    detail: 'No shared datasets yet.',
    icon: Database,
  },
  {
    title: 'Members',
    detail: 'Member access is empty until a team exists.',
    icon: Users,
  },
  {
    title: 'Invitations',
    detail: 'No pending invitations.',
    icon: UserPlus,
  },
];

export function TeamPage() {
  return (
    <div className="space-y-8">
      <section>
        <p className="text-sm font-semibold uppercase tracking-wide text-harvest">
          Shared workspace
        </p>
        <h2 className="mt-2 text-3xl font-semibold tracking-normal sm:text-4xl">Team</h2>
        <p className="mt-3 max-w-3xl text-base leading-7 text-stone-700">
          A secondary tab for collaborative access, shared datasets, invitations, and member roles.
        </p>
      </section>

      <section className="grid gap-4 md:grid-cols-3" aria-label="Team workspace metrics">
        <MetricTile label="Members" value="0" icon={Users} />
        <MetricTile label="Shared datasets" value="0" icon={Database} />
        <MetricTile label="Pending invites" value="0" icon={UserPlus} />
      </section>

      <section className="grid gap-4 lg:grid-cols-[1fr_320px]">
        <div className="grid gap-4 md:grid-cols-3 lg:grid-cols-1">
          {teamSections.map(({ title, detail, icon: Icon }) => (
            <article key={title} className="rounded-md border border-stone-200 bg-white p-5">
              <div className="flex items-start gap-3">
                <Icon className="mt-0.5 h-5 w-5 shrink-0 text-field" aria-hidden="true" />
                <div>
                  <h3 className="text-base font-semibold">{title}</h3>
                  <p className="mt-2 text-sm leading-6 text-stone-600">{detail}</p>
                </div>
              </div>
            </article>
          ))}
        </div>

        <aside className="rounded-md border border-stone-200 bg-white p-5">
          <ShieldCheck className="h-5 w-5 text-field" aria-hidden="true" />
          <h3 className="mt-4 text-base font-semibold">Access boundary</h3>
          <p className="mt-2 text-sm leading-6 text-stone-600">
            Team data remains separate from solo insight work while keeping shared analysis one tab
            away.
          </p>
        </aside>
      </section>
    </div>
  );
}
