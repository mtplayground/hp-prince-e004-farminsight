import {
  AlertTriangle,
  BarChart3,
  CheckCircle2,
  Database,
  Loader2,
  MailPlus,
  RefreshCw,
  Search,
  ShieldCheck,
  Sparkles,
  Table2,
  UserCheck,
  Users,
} from 'lucide-react';
import { useCallback, useMemo, useState, type FormEvent } from 'react';

import {
  type DatasetChartSpec,
  type DatasetInsight,
  type DatasetInsightsResponse,
  type DatasetRecord,
  type DatasetSchemaResponse,
  fetchTeamDataset,
  fetchTeamDatasetInsights,
  fetchTeamDatasetSchema,
} from '../api/datasets';
import {
  createTeamInvitation,
  fetchTeamMembers,
  type TeamInvitation,
  type TeamMember,
} from '../api/teams';
import { useAuth } from '../auth/useAuth';
import { MetricTile } from '../layout/AppShell';

type SharedViewState =
  | { status: 'idle'; error: null; bundle: null }
  | { status: 'loading'; error: null; bundle: null }
  | { status: 'loaded'; error: null; bundle: SharedDatasetBundle }
  | { status: 'error'; error: string; bundle: SharedDatasetBundle | null };

type SharedDatasetBundle = {
  dataset: DatasetRecord;
  schema: DatasetSchemaResponse;
  insights: DatasetInsightsResponse;
};

type MembersState =
  | { status: 'idle'; error: null; members: TeamMember[] }
  | { status: 'loading'; error: null; members: TeamMember[] }
  | { status: 'loaded'; error: null; members: TeamMember[] }
  | { status: 'error'; error: string; members: TeamMember[] };

type InviteState =
  | { status: 'idle'; error: null; invitation: null }
  | { status: 'sending'; error: null; invitation: null }
  | { status: 'sent'; error: null; invitation: TeamInvitation }
  | { status: 'error'; error: string; invitation: TeamInvitation | null };

type SchemaColumn = {
  index: number;
  name: string;
  inferredType: string;
  likelyMeaning: string;
  confidence: number | null;
  evidence: string[];
};

const initialSharedState: SharedViewState = { status: 'idle', error: null, bundle: null };
const initialMembersState: MembersState = { status: 'idle', error: null, members: [] };
const initialInviteState: InviteState = { status: 'idle', error: null, invitation: null };

export function TeamPage() {
  const auth = useAuth();
  const requestedTeamId =
    auth.status === 'authenticated' ? auth.context.team.requested_team_id : null;
  const [teamId, setTeamId] = useState('');
  const [datasetId, setDatasetId] = useState('');
  const [sharedState, setSharedState] = useState<SharedViewState>(initialSharedState);
  const [membersState, setMembersState] = useState<MembersState>(initialMembersState);
  const [inviteState, setInviteState] = useState<InviteState>(initialInviteState);
  const [inviteEmail, setInviteEmail] = useState('');

  const effectiveTeamId = (teamId || requestedTeamId || '').trim();

  const loadSharedDataset = useCallback(
    async (event?: FormEvent<HTMLFormElement>) => {
      event?.preventDefault();
      const nextTeamId = effectiveTeamId;
      const nextDatasetId = datasetId.trim();

      if (!nextTeamId || !nextDatasetId) {
        setSharedState({
          status: 'error',
          error: 'Both team and dataset identifiers are required.',
          bundle: sharedState.bundle,
        });
        return;
      }

      const controller = new AbortController();
      setSharedState({ status: 'loading', error: null, bundle: null });

      try {
        const [dataset, schema, insights] = await Promise.all([
          fetchTeamDataset(nextTeamId, nextDatasetId, controller.signal),
          fetchTeamDatasetSchema(nextTeamId, nextDatasetId, controller.signal),
          fetchTeamDatasetInsights(nextTeamId, nextDatasetId, controller.signal),
        ]);
        setSharedState({
          status: 'loaded',
          error: null,
          bundle: { dataset, schema, insights },
        });
      } catch (cause: unknown) {
        if (controller.signal.aborted) {
          return;
        }
        setSharedState({
          status: 'error',
          error: cause instanceof Error ? cause.message : 'Shared dataset fetch failed',
          bundle: sharedState.bundle,
        });
      }
    },
    [datasetId, effectiveTeamId, sharedState.bundle],
  );

  const loadTeamMembers = useCallback(
    async (event?: FormEvent<HTMLFormElement>) => {
      event?.preventDefault();
      const nextTeamId = effectiveTeamId;

      if (!nextTeamId) {
        setMembersState({
          status: 'error',
          error: 'Team identifier is required.',
          members: membersState.members,
        });
        return;
      }

      const controller = new AbortController();
      setMembersState({ status: 'loading', error: null, members: membersState.members });

      try {
        const response = await fetchTeamMembers(nextTeamId, controller.signal);
        setMembersState({ status: 'loaded', error: null, members: response.members });
      } catch (cause: unknown) {
        if (controller.signal.aborted) {
          return;
        }
        setMembersState({
          status: 'error',
          error: cause instanceof Error ? cause.message : 'Team members fetch failed',
          members: membersState.members,
        });
      }
    },
    [effectiveTeamId, membersState.members],
  );

  const sendInvitation = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      const nextTeamId = effectiveTeamId;
      const email = inviteEmail.trim();

      if (!nextTeamId || !email) {
        setInviteState({
          status: 'error',
          error: 'Team identifier and invite email are required.',
          invitation: inviteState.invitation,
        });
        return;
      }

      const controller = new AbortController();
      setInviteState({ status: 'sending', error: null, invitation: null });

      try {
        const invitation = await createTeamInvitation(nextTeamId, email, controller.signal);
        setInviteState({ status: 'sent', error: null, invitation });
        setInviteEmail('');
      } catch (cause: unknown) {
        if (controller.signal.aborted) {
          return;
        }
        setInviteState({
          status: 'error',
          error: cause instanceof Error ? cause.message : 'Team invitation failed',
          invitation: inviteState.invitation,
        });
      }
    },
    [effectiveTeamId, inviteEmail, inviteState.invitation],
  );

  const bundle = sharedState.bundle;
  const chartSpecs = bundle?.insights.chart_specs ?? [];
  const memberCount = membersState.members.length;

  return (
    <div className="space-y-8">
      <section className="grid gap-6 xl:grid-cols-[1fr_360px]">
        <div>
          <p className="text-sm font-semibold uppercase tracking-wide text-harvest">
            Shared workspace
          </p>
          <h2 className="mt-2 text-3xl font-semibold tracking-normal sm:text-4xl">Team</h2>
          <p className="mt-3 max-w-3xl text-base leading-7 text-stone-700">
            A secondary view for team-scoped datasets, shared insight summaries, and the chart
            recommendations already generated from the solo workflow.
          </p>
        </div>
        <aside className="rounded-md border border-stone-200 bg-white p-5">
          <ShieldCheck className="h-5 w-5 text-field" aria-hidden="true" />
          <h3 className="mt-4 text-base font-semibold">Access boundary</h3>
          <p className="mt-2 text-sm leading-6 text-stone-600">
            Shared dataset requests are served through team-scoped API routes and return data only
            for memberships attached to the requested team.
          </p>
        </aside>
      </section>

      <section className="grid gap-4 md:grid-cols-3" aria-label="Team workspace metrics">
        <MetricTile label="Members" value={memberCount.toLocaleString()} icon={Users} />
        <MetricTile label="Shared dataset" value={bundle ? '1' : '0'} icon={Database} />
        <MetricTile
          label="Chart specs"
          value={chartSpecs.length.toLocaleString()}
          icon={BarChart3}
        />
      </section>

      <section className="grid gap-4 lg:grid-cols-[360px_1fr]">
        <SharedDatasetLoader
          teamId={teamId}
          datasetId={datasetId}
          status={sharedState.status}
          error={sharedState.error}
          onTeamIdChange={setTeamId}
          onDatasetIdChange={setDatasetId}
          teamIdFallback={requestedTeamId}
          onSubmit={loadSharedDataset}
        />

        <MemberManagementPanel
          effectiveTeamId={effectiveTeamId}
          membersState={membersState}
          inviteEmail={inviteEmail}
          inviteState={inviteState}
          onInviteEmailChange={setInviteEmail}
          onLoadMembers={loadTeamMembers}
          onInvite={sendInvitation}
        />
      </section>

      <SharedDatasetOverview bundle={bundle} status={sharedState.status} />

      {bundle && (
        <>
          <SharedInsightSummary insights={bundle.insights.insights} dataset={bundle.dataset} />
          <SharedChartSpecs chartSpecs={bundle.insights.chart_specs} />
          <SharedSchemaDetail schema={bundle.schema} />
        </>
      )}
    </div>
  );
}

function MemberManagementPanel({
  effectiveTeamId,
  membersState,
  inviteEmail,
  inviteState,
  onInviteEmailChange,
  onLoadMembers,
  onInvite,
}: {
  effectiveTeamId: string;
  membersState: MembersState;
  inviteEmail: string;
  inviteState: InviteState;
  onInviteEmailChange: (value: string) => void;
  onLoadMembers: (event?: FormEvent<HTMLFormElement>) => void;
  onInvite: (event: FormEvent<HTMLFormElement>) => void;
}) {
  return (
    <section className="rounded-md border border-stone-200 bg-white p-5">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <Users className="h-5 w-5 text-field" aria-hidden="true" />
            <p className="text-sm font-semibold uppercase tracking-wide text-harvest">
              Member management
            </p>
          </div>
          <h3 className="mt-2 text-xl font-semibold text-stone-950">Owners and members</h3>
          <p className="mt-2 text-sm leading-6 text-stone-600">
            Invite collaborators and review the current team roster.
          </p>
        </div>

        <button
          type="button"
          onClick={() => onLoadMembers()}
          disabled={!effectiveTeamId || membersState.status === 'loading'}
          className="inline-flex min-h-10 items-center justify-center gap-2 rounded-md border border-stone-200 px-3 text-sm font-semibold text-stone-700 hover:bg-stone-100 disabled:cursor-not-allowed disabled:text-stone-400"
        >
          {membersState.status === 'loading' ? (
            <Loader2 className="h-4 w-4 animate-spin" aria-hidden="true" />
          ) : (
            <RefreshCw className="h-4 w-4" aria-hidden="true" />
          )}
          Refresh
        </button>
      </div>

      <form onSubmit={onInvite} className="mt-5 grid gap-3 md:grid-cols-[1fr_auto]">
        <label className="block">
          <span className="text-xs font-semibold uppercase tracking-wide text-stone-500">
            Invite email
          </span>
          <input
            value={inviteEmail}
            onChange={(event) => onInviteEmailChange(event.currentTarget.value)}
            className="mt-2 h-10 w-full rounded-md border border-stone-300 bg-white px-3 text-sm text-stone-900"
            placeholder="member@example.com"
            type="email"
          />
        </label>

        <button
          type="submit"
          disabled={!effectiveTeamId || inviteState.status === 'sending'}
          className="mt-0 inline-flex min-h-10 items-center justify-center gap-2 rounded-md bg-field px-4 text-sm font-semibold text-white disabled:cursor-not-allowed disabled:bg-stone-300 disabled:text-stone-600 md:mt-6"
        >
          {inviteState.status === 'sending' ? (
            <Loader2 className="h-4 w-4 animate-spin" aria-hidden="true" />
          ) : (
            <MailPlus className="h-4 w-4" aria-hidden="true" />
          )}
          Invite
        </button>
      </form>

      {membersState.error && (
        <StatusCallout tone="warning" message={membersState.error} icon={AlertTriangle} />
      )}

      {inviteState.error && (
        <StatusCallout tone="warning" message={inviteState.error} icon={AlertTriangle} />
      )}

      {inviteState.status === 'sent' && (
        <StatusCallout
          tone="success"
          message={`Invitation queued for ${inviteState.invitation.email}. Delivery status: ${kindLabel(inviteState.invitation.email_delivery.status)}.`}
          icon={CheckCircle2}
        />
      )}

      <div className="mt-5 overflow-hidden rounded-md border border-stone-200">
        <div className="flex items-center justify-between gap-3 border-b border-stone-200 bg-stone-50 px-3 py-2">
          <div className="flex items-center gap-2">
            <UserCheck className="h-4 w-4 text-field" aria-hidden="true" />
            <p className="text-sm font-semibold text-stone-800">Team roster</p>
          </div>
          <p className="text-xs text-stone-500">
            {membersState.members.length.toLocaleString()} shown
          </p>
        </div>

        <div className="divide-y divide-stone-100 bg-white">
          {membersState.members.map((member) => (
            <MemberRow key={member.user_sub} member={member} />
          ))}

          {membersState.members.length === 0 && (
            <div className="px-4 py-8 text-center text-sm text-stone-500">
              No members loaded for this team.
            </div>
          )}
        </div>
      </div>
    </section>
  );
}

function MemberRow({ member }: { member: TeamMember }) {
  const label = member.name || member.email;
  const initial = label.trim().charAt(0).toUpperCase() || '?';

  return (
    <article className="flex items-center gap-3 px-4 py-3">
      {member.picture_url ? (
        <img
          src={member.picture_url}
          alt=""
          className="h-10 w-10 shrink-0 rounded-md border border-stone-200 object-cover"
        />
      ) : (
        <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-field text-sm font-semibold text-white">
          {initial}
        </span>
      )}
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-semibold text-stone-950">{label}</p>
        <p className="truncate text-xs text-stone-600">{member.email}</p>
      </div>
      <div className="text-right">
        <p className="text-xs font-semibold uppercase tracking-wide text-harvest">
          {kindLabel(member.role)}
        </p>
        <p className="mt-1 text-xs text-stone-500">Seen {formatDate(member.last_seen_at)}</p>
      </div>
    </article>
  );
}

function StatusCallout({
  tone,
  message,
  icon: Icon,
}: {
  tone: 'success' | 'warning';
  message: string;
  icon: typeof CheckCircle2;
}) {
  const className =
    tone === 'success'
      ? 'border-green-200 bg-green-50 text-green-800'
      : 'border-amber-200 bg-amber-50 text-amber-800';

  return (
    <div className={`mt-4 flex gap-2 rounded-md border p-3 text-sm ${className}`}>
      <Icon className="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
      <p>{message}</p>
    </div>
  );
}

function SharedDatasetLoader({
  teamId,
  datasetId,
  status,
  error,
  onTeamIdChange,
  onDatasetIdChange,
  teamIdFallback,
  onSubmit,
}: {
  teamId: string;
  datasetId: string;
  status: SharedViewState['status'];
  error: string | null;
  onTeamIdChange: (value: string) => void;
  onDatasetIdChange: (value: string) => void;
  teamIdFallback: string | null;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
}) {
  return (
    <form onSubmit={onSubmit} className="rounded-md border border-stone-200 bg-white p-5">
      <div className="flex items-start gap-3">
        <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-field text-white">
          <Search className="h-5 w-5" aria-hidden="true" />
        </span>
        <div>
          <h3 className="text-lg font-semibold">Shared dataset</h3>
          <p className="mt-1 text-sm leading-6 text-stone-600">
            Load a dataset through the team access API.
          </p>
        </div>
      </div>

      <div className="mt-5 space-y-4">
        <label className="block">
          <span className="text-xs font-semibold uppercase tracking-wide text-stone-500">
            Team ID
          </span>
          <input
            value={teamId}
            onChange={(event) => onTeamIdChange(event.currentTarget.value)}
            className="mt-2 h-10 w-full rounded-md border border-stone-300 bg-white px-3 text-sm text-stone-900"
            placeholder={teamIdFallback ?? 'Team UUID'}
          />
        </label>

        <label className="block">
          <span className="text-xs font-semibold uppercase tracking-wide text-stone-500">
            Dataset ID
          </span>
          <input
            value={datasetId}
            onChange={(event) => onDatasetIdChange(event.currentTarget.value)}
            className="mt-2 h-10 w-full rounded-md border border-stone-300 bg-white px-3 text-sm text-stone-900"
            placeholder="Dataset UUID"
          />
        </label>
      </div>

      <button
        type="submit"
        disabled={status === 'loading'}
        className="mt-5 inline-flex min-h-10 w-full items-center justify-center gap-2 rounded-md bg-field px-3 text-sm font-semibold text-white disabled:cursor-not-allowed disabled:bg-stone-300 disabled:text-stone-600"
      >
        {status === 'loading' ? (
          <Loader2 className="h-4 w-4 animate-spin" aria-hidden="true" />
        ) : (
          <Search className="h-4 w-4" aria-hidden="true" />
        )}
        Load shared view
      </button>

      {error && (
        <div className="mt-4 flex gap-2 rounded-md border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800">
          <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
          <p>{error}</p>
        </div>
      )}
    </form>
  );
}

function SharedDatasetOverview({
  bundle,
  status,
}: {
  bundle: SharedDatasetBundle | null;
  status: SharedViewState['status'];
}) {
  if (status === 'loading') {
    return (
      <section className="flex min-h-64 items-center justify-center rounded-md border border-stone-200 bg-white text-stone-600">
        <Loader2 className="mr-2 h-5 w-5 animate-spin" aria-hidden="true" />
        Loading shared dataset
      </section>
    );
  }

  if (!bundle) {
    return (
      <section className="rounded-md border border-dashed border-stone-300 bg-white p-6">
        <div className="flex items-start gap-3">
          <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-field text-white">
            <Database className="h-5 w-5" aria-hidden="true" />
          </span>
          <div>
            <h3 className="text-lg font-semibold">No shared dataset selected</h3>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-stone-600">
              Team details appear here after a shared dataset is loaded.
            </p>
          </div>
        </div>
      </section>
    );
  }

  const { dataset, schema, insights } = bundle;

  return (
    <section className="rounded-md border border-stone-200 bg-white p-6">
      <div className="flex flex-col gap-5 xl:flex-row xl:items-start xl:justify-between">
        <div className="min-w-0">
          <p className="text-sm font-semibold uppercase tracking-wide text-harvest">
            Shared dataset detail
          </p>
          <h3 className="mt-2 break-words text-2xl font-semibold tracking-normal text-stone-950">
            {dataset.original_filename}
          </h3>
          <p className="mt-2 text-sm leading-6 text-stone-600">
            Uploaded {formatDate(dataset.uploaded_at)} by {dataset.owner_sub}
          </p>
        </div>

        <div className="grid min-w-64 grid-cols-2 gap-2">
          <DetailStat label="Rows" value={formatNullableNumber(schema.row_count)} />
          <DetailStat label="Columns" value={formatNullableNumber(schema.column_count)} />
          <DetailStat label="Findings" value={insights.insights.length.toLocaleString()} />
          <DetailStat label="Charts" value={insights.chart_specs.length.toLocaleString()} />
        </div>
      </div>

      <div className="mt-5 grid gap-3 md:grid-cols-2">
        <ReferenceLine label="Team ID" value={dataset.team_id ?? 'No team scope'} />
        <ReferenceLine label="Dataset ID" value={dataset.id} />
      </div>
    </section>
  );
}

function SharedInsightSummary({
  insights,
  dataset,
}: {
  insights: DatasetInsight[];
  dataset: DatasetRecord;
}) {
  const topInsight = insights[0] ?? null;
  const supportingInsights = insights.slice(1, 4);

  return (
    <section className="rounded-md border border-field/20 bg-white p-6 shadow-sm shadow-green-900/5">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="max-w-3xl">
          <div className="flex items-center gap-2">
            <Sparkles className="h-5 w-5 text-field" aria-hidden="true" />
            <p className="text-sm font-semibold uppercase tracking-wide text-harvest">
              Shared summary
            </p>
          </div>
          <h3 className="mt-2 text-2xl font-semibold tracking-normal text-stone-950">
            {topInsight?.title ?? 'No shared summary available'}
          </h3>
          <p className="mt-3 text-base leading-7 text-stone-700">
            {topInsight?.summary ??
              `${dataset.original_filename} has no generated plain-language findings yet.`}
          </p>
        </div>

        <DetailStat
          label="Confidence"
          value={topInsight ? `${Math.round(topInsight.confidence * 100)}%` : 'Pending'}
        />
      </div>

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
    </section>
  );
}

function SharedChartSpecs({ chartSpecs }: { chartSpecs: DatasetChartSpec[] }) {
  if (chartSpecs.length === 0) {
    return (
      <section className="rounded-md border border-stone-200 bg-white p-6">
        <div className="flex items-start gap-3">
          <BarChart3 className="mt-0.5 h-5 w-5 text-field" aria-hidden="true" />
          <div>
            <h3 className="text-lg font-semibold">Shared chart specs</h3>
            <p className="mt-2 text-sm leading-6 text-stone-600">
              No chart recommendations are cached for this dataset.
            </p>
          </div>
        </div>
      </section>
    );
  }

  return (
    <section className="space-y-4" aria-label="Shared chart specs">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-sm font-semibold uppercase tracking-wide text-harvest">
            Shared chart specs
          </p>
          <h3 className="mt-2 text-2xl font-semibold tracking-normal text-stone-950">
            Recommendations
          </h3>
        </div>
        <p className="text-sm text-stone-600">
          {chartSpecs.length.toLocaleString()} chart {chartSpecs.length === 1 ? 'spec' : 'specs'}
        </p>
      </div>

      <div className="grid gap-4 xl:grid-cols-2">
        {chartSpecs.map((spec) => (
          <article key={spec.id} className="rounded-md border border-stone-200 bg-white p-5">
            <div className="flex items-start justify-between gap-4">
              <div>
                <p className="text-xs font-semibold uppercase tracking-wide text-harvest">
                  {kindLabel(spec.chart_type)}
                </p>
                <h4 className="mt-2 text-base font-semibold text-stone-950">{spec.title}</h4>
                <p className="mt-2 text-sm leading-6 text-stone-600">{spec.rationale}</p>
              </div>
              <span className="rounded-md border border-stone-200 bg-stone-50 px-2 py-1 text-xs font-semibold text-stone-600">
                {Math.round(spec.confidence * 100)}%
              </span>
            </div>

            <div className="mt-4 grid gap-2 sm:grid-cols-3">
              <ReferenceLine label="X" value={spec.x.name} />
              <ReferenceLine label="Y" value={spec.y.name} />
              <ReferenceLine label="Series" value={spec.series?.name ?? 'None'} />
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function SharedSchemaDetail({ schema }: { schema: DatasetSchemaResponse }) {
  const columns = useMemo(() => schemaColumns(schema), [schema]);
  const statsByIndex = useMemo(() => statsLookup(schema.column_stats), [schema.column_stats]);

  return (
    <section className="rounded-md border border-stone-200 bg-white p-6">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <Table2 className="h-5 w-5 text-field" aria-hidden="true" />
            <p className="text-sm font-semibold uppercase tracking-wide text-harvest">
              Shared schema
            </p>
          </div>
          <h3 className="mt-2 text-2xl font-semibold tracking-normal text-stone-950">
            Column profile
          </h3>
        </div>
        <p className="text-sm text-stone-600">
          {columns.length.toLocaleString()} inferred {columns.length === 1 ? 'column' : 'columns'}
        </p>
      </div>

      <div className="mt-5 overflow-hidden rounded-md border border-stone-200">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-stone-200 text-left text-sm">
            <thead className="bg-stone-50">
              <tr>
                <th
                  scope="col"
                  className="whitespace-nowrap px-3 py-2 font-semibold text-stone-700"
                >
                  Column
                </th>
                <th
                  scope="col"
                  className="whitespace-nowrap px-3 py-2 font-semibold text-stone-700"
                >
                  Type
                </th>
                <th
                  scope="col"
                  className="whitespace-nowrap px-3 py-2 font-semibold text-stone-700"
                >
                  Meaning
                </th>
                <th
                  scope="col"
                  className="whitespace-nowrap px-3 py-2 font-semibold text-stone-700"
                >
                  Stats
                </th>
                <th
                  scope="col"
                  className="whitespace-nowrap px-3 py-2 font-semibold text-stone-700"
                >
                  Evidence
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-100 bg-white">
              {columns.map((column) => {
                const stats = statsByIndex.get(column.index);
                return (
                  <tr key={`${column.index}-${column.name}`}>
                    <td className="max-w-64 px-3 py-3 font-semibold text-stone-900">
                      <span className="block truncate">{column.name}</span>
                      <span className="mt-1 block text-xs font-normal text-stone-500">
                        Confidence{' '}
                        {column.confidence === null
                          ? 'pending'
                          : `${Math.round(column.confidence * 100)}%`}
                      </span>
                    </td>
                    <td className="whitespace-nowrap px-3 py-3 text-stone-700">
                      {kindLabel(column.inferredType)}
                    </td>
                    <td className="whitespace-nowrap px-3 py-3 text-stone-700">
                      {kindLabel(column.likelyMeaning)}
                    </td>
                    <td className="min-w-48 px-3 py-3 text-stone-700">
                      {stats ? statSummary(stats) : 'No stats'}
                    </td>
                    <td className="min-w-72 px-3 py-3 text-stone-600">
                      {column.evidence[0] ?? 'No evidence recorded'}
                    </td>
                  </tr>
                );
              })}
              {columns.length === 0 && (
                <tr>
                  <td className="px-3 py-6 text-center text-sm text-stone-500" colSpan={5}>
                    No schema columns are cached for this dataset.
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

function DetailStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-stone-200 bg-stone-50 p-3">
      <p className="text-xs font-medium text-stone-500">{label}</p>
      <p className="mt-1 break-words text-lg font-semibold text-stone-950">{value}</p>
    </div>
  );
}

function ReferenceLine({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-stone-200 bg-stone-50 p-3">
      <p className="text-xs font-semibold uppercase tracking-wide text-stone-500">{label}</p>
      <p className="mt-1 break-all text-sm font-semibold text-stone-900">{value}</p>
    </div>
  );
}

function schemaColumns(schema: DatasetSchemaResponse) {
  const rawColumns = schema.detected_schema.columns;
  if (!Array.isArray(rawColumns)) {
    return schema.column_names.map((name, index) => ({
      index,
      name,
      inferredType: 'unknown',
      likelyMeaning: 'unknown',
      confidence: null,
      evidence: [],
    }));
  }

  return rawColumns
    .map((rawColumn): SchemaColumn | null => {
      if (!isRecord(rawColumn)) {
        return null;
      }

      const index = numberValue(rawColumn.index);
      const name = stringValue(rawColumn.name);
      if (index === null || !name) {
        return null;
      }

      return {
        index,
        name,
        inferredType: stringValue(rawColumn.inferred_type) || 'unknown',
        likelyMeaning: stringValue(rawColumn.likely_meaning) || 'unknown',
        confidence: numberValue(rawColumn.confidence),
        evidence: stringArray(rawColumn.evidence),
      };
    })
    .filter((column): column is SchemaColumn => column !== null);
}

function statsLookup(stats: Record<string, unknown>[]) {
  const lookup = new Map<number, Record<string, unknown>>();

  stats.forEach((stat) => {
    const index = numberValue(stat.index);
    if (index !== null) {
      lookup.set(index, stat);
    }
  });

  return lookup;
}

function statSummary(stat: Record<string, unknown>) {
  const nonEmpty = numberValue(stat.non_empty_count);
  const blank = numberValue(stat.blank_count);
  const unique = numberValue(stat.unique_count);
  const samples = stringArray(stat.sample_values).slice(0, 2);
  const pieces = [
    nonEmpty === null ? null : `${nonEmpty.toLocaleString()} filled`,
    blank === null ? null : `${blank.toLocaleString()} blank`,
    unique === null ? null : `${unique.toLocaleString()} unique`,
  ].filter((piece): piece is string => piece !== null);

  if (samples.length > 0) {
    pieces.push(`Samples: ${samples.join(', ')}`);
  }

  return pieces.join(' · ') || 'No stats';
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function numberValue(value: unknown) {
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

function stringValue(value: unknown) {
  return typeof value === 'string' ? value : null;
}

function stringArray(value: unknown) {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === 'string')
    : [];
}

function kindLabel(kind: string) {
  return kind.replaceAll('_', ' ');
}

function formatNullableNumber(value: number | null) {
  return value === null ? 'Unknown' : value.toLocaleString();
}

function formatDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}
