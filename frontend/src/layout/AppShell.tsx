import {
  Activity,
  BarChart3,
  ChevronRight,
  CircleGauge,
  Database,
  LogIn,
  PanelLeft,
  UserPlus,
  Users,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { NavLink, Outlet, useLocation } from 'react-router-dom';

import { useAuth } from '../auth/useAuth';

type HealthState =
  | { status: 'checking' }
  | { status: 'ready'; service: string; version: string; database: string }
  | { status: 'unavailable'; message: string };

const navItems = [
  {
    label: 'Insights',
    to: '/insights',
    icon: BarChart3,
    description: 'Solo analysis',
  },
  {
    label: 'Team',
    to: '/team',
    icon: Users,
    description: 'Shared workspace',
  },
];

const routeTitles: Record<string, string> = {
  '/insights': 'Insights',
  '/team': 'Team',
  '/login': 'Sign in',
  '/register': 'Create account',
};

export function AppShell() {
  const health = useApiHealth();
  const auth = useAuth();
  const location = useLocation();
  const routeTitle = routeTitles[location.pathname] ?? 'Workspace';

  return (
    <div className="min-h-screen bg-stone-50 text-ink">
      <header className="sticky top-0 z-20 border-b border-stone-200 bg-white/95 backdrop-blur">
        <div className="flex min-h-16 items-center justify-between px-4 sm:px-6 lg:hidden">
          <div>
            <p className="text-xs font-semibold uppercase tracking-wide text-field">
              CSV analytics
            </p>
            <h1 className="text-lg font-semibold">Farminsight</h1>
          </div>
          <div className="flex items-center gap-2">
            <AccountControl compact />
            <StatusPill health={health} compact />
          </div>
        </div>
        <nav className="flex gap-1 overflow-x-auto border-t border-stone-100 px-3 py-2 lg:hidden">
          {navItems.map(({ label, to, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) =>
                [
                  'inline-flex min-h-10 flex-1 items-center justify-center gap-2 rounded-md px-3 text-sm font-semibold',
                  isActive
                    ? 'bg-field text-white'
                    : 'text-stone-700 hover:bg-stone-100 hover:text-ink',
                ].join(' ')
              }
            >
              <Icon className="h-4 w-4" aria-hidden="true" />
              {label}
            </NavLink>
          ))}
        </nav>
      </header>

      <div className="mx-auto grid max-w-7xl lg:grid-cols-[272px_1fr]">
        <aside className="sticky top-0 hidden h-screen border-r border-stone-200 bg-white lg:block">
          <div className="flex h-full flex-col">
            <div className="border-b border-stone-200 px-6 py-5">
              <p className="text-xs font-semibold uppercase tracking-wide text-field">
                CSV analytics
              </p>
              <h1 className="mt-1 text-2xl font-semibold">Farminsight</h1>
            </div>

            <nav className="space-y-1 px-3 py-4" aria-label="Primary">
              {navItems.map(({ label, to, icon: Icon, description }) => (
                <NavLink
                  key={to}
                  to={to}
                  className={({ isActive }) =>
                    [
                      'group flex min-h-14 items-center gap-3 rounded-md px-3 py-2',
                      isActive
                        ? 'bg-field text-white'
                        : 'text-stone-700 hover:bg-stone-100 hover:text-ink',
                    ].join(' ')
                  }
                >
                  <Icon className="h-5 w-5 shrink-0" aria-hidden="true" />
                  <span className="min-w-0 flex-1">
                    <span className="block text-sm font-semibold">{label}</span>
                    <span className="block truncate text-xs opacity-80">{description}</span>
                  </span>
                  <ChevronRight className="h-4 w-4 opacity-70" aria-hidden="true" />
                </NavLink>
              ))}
            </nav>

            <div className="border-t border-stone-200 px-3 py-4">
              {auth.status === 'authenticated' ? (
                <UserSummary
                  name={auth.context.session.user.name}
                  email={auth.context.session.user.email}
                  pictureUrl={auth.context.session.user.picture_url}
                />
              ) : (
                <AuthLinks />
              )}
            </div>

            <div className="mt-auto border-t border-stone-200 p-4">
              <RuntimePanel health={health} />
            </div>
          </div>
        </aside>

        <main className="min-w-0">
          <div className="hidden h-16 items-center justify-between border-b border-stone-200 bg-white px-8 lg:flex">
            <div className="flex items-center gap-2 text-sm text-stone-600">
              <PanelLeft className="h-4 w-4" aria-hidden="true" />
              <span>Workspace</span>
              <ChevronRight className="h-4 w-4" aria-hidden="true" />
              <span className="font-semibold text-ink">{routeTitle}</span>
            </div>
            <StatusPill health={health} />
          </div>

          <div className="px-4 py-5 sm:px-6 lg:px-8 lg:py-8">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
}

function AccountControl({ compact = false }: { compact?: boolean }) {
  const auth = useAuth();

  if (auth.status === 'authenticated') {
    return (
      <UserAvatar
        name={auth.context.session.user.name}
        email={auth.context.session.user.email}
        pictureUrl={auth.context.session.user.picture_url}
        compact={compact}
      />
    );
  }

  return <AuthLinks compact={compact} />;
}

function AuthLinks({ compact = false }: { compact?: boolean }) {
  if (compact) {
    return (
      <NavLink
        to="/login"
        className={({ isActive }) =>
          [
            'inline-flex min-h-9 items-center justify-center rounded-md px-2.5 text-sm font-semibold',
            isActive ? 'bg-field text-white' : 'text-stone-700 hover:bg-stone-100 hover:text-ink',
          ].join(' ')
        }
        aria-label="Sign in"
      >
        <LogIn className="h-4 w-4" aria-hidden="true" />
      </NavLink>
    );
  }

  return (
    <div className="grid grid-cols-2 gap-2">
      <NavLink
        to="/login"
        className={({ isActive }) =>
          [
            'inline-flex min-h-10 items-center justify-center gap-2 rounded-md px-3 text-sm font-semibold',
            isActive
              ? 'bg-field text-white'
              : 'border border-stone-200 text-stone-700 hover:bg-stone-100 hover:text-ink',
          ].join(' ')
        }
      >
        <LogIn className="h-4 w-4" aria-hidden="true" />
        Sign in
      </NavLink>
      <NavLink
        to="/register"
        className={({ isActive }) =>
          [
            'inline-flex min-h-10 items-center justify-center gap-2 rounded-md px-3 text-sm font-semibold',
            isActive ? 'bg-field text-white' : 'bg-stone-900 text-white hover:bg-stone-700',
          ].join(' ')
        }
      >
        <UserPlus className="h-4 w-4" aria-hidden="true" />
        Register
      </NavLink>
    </div>
  );
}

function UserSummary({
  name,
  email,
  pictureUrl,
}: {
  name: string | null;
  email: string;
  pictureUrl: string | null;
}) {
  return (
    <div className="flex items-center gap-3 rounded-md border border-stone-200 bg-stone-50 p-3">
      <UserAvatar name={name} email={email} pictureUrl={pictureUrl} />
      <div className="min-w-0">
        <p className="truncate text-sm font-semibold">{name || email}</p>
        <p className="truncate text-xs text-stone-600">{email}</p>
      </div>
    </div>
  );
}

function UserAvatar({
  name,
  email,
  pictureUrl,
  compact = false,
}: {
  name: string | null;
  email: string;
  pictureUrl: string | null;
  compact?: boolean;
}) {
  const label = name || email;
  const initial = label.trim().charAt(0).toUpperCase() || '?';
  const size = compact ? 'h-9 w-9' : 'h-10 w-10';

  if (pictureUrl) {
    return (
      <img
        src={pictureUrl}
        alt=""
        className={`${size} shrink-0 rounded-md border border-stone-200 object-cover`}
      />
    );
  }

  return (
    <span
      className={`${size} inline-flex shrink-0 items-center justify-center rounded-md bg-field text-sm font-semibold text-white`}
      aria-label={label}
    >
      {initial}
    </span>
  );
}

function useApiHealth(): HealthState {
  const [health, setHealth] = useState<HealthState>({ status: 'checking' });

  useEffect(() => {
    const controller = new AbortController();

    fetch('/api/health', { signal: controller.signal })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`API returned ${response.status}`);
        }
        return response.json() as Promise<{ service: string; version: string; database: string }>;
      })
      .then((body) => {
        setHealth({
          status: 'ready',
          service: body.service,
          version: body.version,
          database: body.database,
        });
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

  return health;
}

function RuntimePanel({ health }: { health: HealthState }) {
  return (
    <section
      aria-label="Runtime status"
      className="rounded-md border border-stone-200 bg-stone-50 p-4"
    >
      <div className="flex items-center gap-3">
        <span className="flex h-9 w-9 items-center justify-center rounded-md bg-white text-field shadow-sm">
          <Activity className="h-4 w-4" aria-hidden="true" />
        </span>
        <div>
          <p className="text-sm font-semibold">Runtime</p>
          <p className="text-xs text-stone-600">Axum API and SPA host</p>
        </div>
      </div>
      <div className="mt-4">
        <StatusPill health={health} />
      </div>
      {health.status === 'ready' && (
        <dl className="mt-4 grid grid-cols-2 gap-3 text-xs">
          <div>
            <dt className="text-stone-500">Service</dt>
            <dd className="mt-1 truncate font-semibold">{health.service}</dd>
          </div>
          <div>
            <dt className="text-stone-500">Database</dt>
            <dd className="mt-1 font-semibold">{health.database}</dd>
          </div>
        </dl>
      )}
      {health.status === 'unavailable' && (
        <p className="mt-3 break-words text-xs leading-5 text-stone-600">{health.message}</p>
      )}
    </section>
  );
}

function StatusPill({ health, compact = false }: { health: HealthState; compact?: boolean }) {
  const tone = {
    checking: 'bg-stone-100 text-stone-700',
    ready: 'bg-green-50 text-green-700',
    unavailable: 'bg-amber-50 text-amber-700',
  }[health.status];

  const label =
    health.status === 'ready'
      ? compact
        ? 'Ready'
        : `Ready v${health.version}`
      : statusLabel(health.status);

  return (
    <span
      className={`inline-flex min-h-8 items-center gap-2 rounded-md px-2.5 text-xs font-semibold ${tone}`}
    >
      <CircleGauge className="h-3.5 w-3.5" aria-hidden="true" />
      {label}
    </span>
  );
}

function statusLabel(status: Exclude<HealthState['status'], 'ready'>) {
  return {
    checking: 'Checking',
    unavailable: 'Offline',
  }[status];
}

export function MetricTile({
  label,
  value,
  icon: Icon,
}: {
  label: string;
  value: string;
  icon: typeof Database;
}) {
  return (
    <div className="rounded-md border border-stone-200 bg-white p-4">
      <div className="flex items-center justify-between gap-3">
        <p className="text-sm font-medium text-stone-600">{label}</p>
        <Icon className="h-4 w-4 shrink-0 text-field" aria-hidden="true" />
      </div>
      <p className="mt-3 text-2xl font-semibold">{value}</p>
    </div>
  );
}
