import { expect, test, type Page, type Route } from '@playwright/test';

const csv = [
  'Season,Field,Crop,Yield,Rainfall',
  '2024,North,Corn,180,21',
  '2025,North,Corn,210,19',
  '2025,South,Soy,145,26',
].join('\n');

const preview = {
  columns: ['Season', 'Field', 'Crop', 'Yield', 'Rainfall'],
  rows: [
    ['2024', 'North', 'Corn', '180', '21'],
    ['2025', 'North', 'Corn', '210', '19'],
    ['2025', 'South', 'Soy', '145', '26'],
  ],
  row_count: 3,
  column_count: 5,
  delimiter: ',',
  truncated: false,
  warnings: [],
};

const insights = [
  {
    kind: 'dataset_shape',
    title: 'Dataset is ready for a quick scan',
    summary:
      'I found 3 rows and 5 columns, including 2 numeric measures and 2 likely grouping fields.',
    evidence: ["Parser used ',' as the delimiter and kept 3 preview rows."],
    confidence: 0.82,
  },
  {
    kind: 'trend',
    title: 'Yield increased over time',
    summary: 'Yield increased from 180 to 210 between 2024 and 2025.',
    evidence: ['Compared average values using Season as the date field.'],
    confidence: 0.86,
  },
];

const chartSpecs = [
  {
    id: 'line-0-3',
    chart_type: 'line',
    title: 'Yield over Season',
    rationale: 'Season looks like a time field and Yield is numeric.',
    x: { index: 0, name: 'Season' },
    y: { index: 3, name: 'Yield' },
    series: { index: 1, name: 'Field' },
    aggregation: 'average',
    confidence: 0.88,
  },
  {
    id: 'bar-1-3',
    chart_type: 'bar',
    title: 'Average Yield by Field',
    rationale: 'Field groups the numeric Yield values.',
    x: { index: 1, name: 'Field' },
    y: { index: 3, name: 'Yield' },
    series: null,
    aggregation: 'average',
    confidence: 0.84,
  },
  {
    id: 'scatter-4-3',
    chart_type: 'scatter',
    title: 'Yield vs Rainfall',
    rationale: 'Rainfall and Yield have enough numeric values for a relationship check.',
    x: { index: 4, name: 'Rainfall' },
    y: { index: 3, name: 'Yield' },
    series: null,
    aggregation: null,
    confidence: 0.74,
  },
];

const detectedSchema = {
  inference_version: 1,
  columns: [
    {
      index: 0,
      name: 'Season',
      inferred_type: 'date',
      likely_meaning: 'date',
      confidence: 0.9,
      evidence: ['Column name suggests a date or time field.'],
    },
    {
      index: 1,
      name: 'Field',
      inferred_type: 'categorical',
      likely_meaning: 'field_name',
      confidence: 0.86,
      evidence: ['Column name matches field or farm naming language.'],
    },
    {
      index: 2,
      name: 'Crop',
      inferred_type: 'categorical',
      likely_meaning: 'crop_type',
      confidence: 0.83,
      evidence: ['Column name matches crop classification language.'],
    },
    {
      index: 3,
      name: 'Yield',
      inferred_type: 'numeric',
      likely_meaning: 'numeric_trend',
      confidence: 0.9,
      evidence: ['Most sampled values parsed as numbers.'],
    },
    {
      index: 4,
      name: 'Rainfall',
      inferred_type: 'numeric',
      likely_meaning: 'numeric_trend',
      confidence: 0.78,
      evidence: ['Most sampled values parsed as numbers.'],
    },
  ],
};

const columnStats = [
  {
    index: 0,
    name: 'Season',
    non_empty_count: 3,
    blank_count: 0,
    unique_count: 2,
    sample_values: ['2024', '2025'],
  },
  {
    index: 1,
    name: 'Field',
    non_empty_count: 3,
    blank_count: 0,
    unique_count: 2,
    sample_values: ['North', 'South'],
  },
  {
    index: 2,
    name: 'Crop',
    non_empty_count: 3,
    blank_count: 0,
    unique_count: 2,
    sample_values: ['Corn', 'Soy'],
  },
  {
    index: 3,
    name: 'Yield',
    non_empty_count: 3,
    blank_count: 0,
    unique_count: 3,
    sample_values: ['180', '210'],
  },
  {
    index: 4,
    name: 'Rainfall',
    non_empty_count: 3,
    blank_count: 0,
    unique_count: 3,
    sample_values: ['21', '19'],
  },
];

const dataset = {
  id: 'dataset-1',
  owner_sub: 'user-owner',
  team_id: 'team-1',
  original_filename: 'farm-yields.csv',
  storage: {
    bucket: 'test-bucket',
    key: 'prefix/datasets/user-owner/dataset-1/farm-yields.csv',
    content_type: 'text/csv',
    byte_size: csv.length,
  },
  row_count: 3,
  column_count: 5,
  column_names: preview.columns,
  detected_schema: detectedSchema,
  column_stats: columnStats,
  cached_insights: insights,
  cached_chart_specs: chartSpecs,
  stats: { parser: 'forgiving', raw_csv: true },
  uploaded_at: '2026-07-21T00:00:00Z',
  created_at: '2026-07-21T00:00:00Z',
  updated_at: '2026-07-21T00:00:00Z',
};

const schemaResponse = {
  dataset_id: dataset.id,
  owner_sub: dataset.owner_sub,
  team_id: dataset.team_id,
  original_filename: dataset.original_filename,
  row_count: dataset.row_count,
  column_count: dataset.column_count,
  column_names: dataset.column_names,
  detected_schema: detectedSchema,
  column_stats: columnStats,
  cached_insights: insights,
  cached_chart_specs: chartSpecs,
  stats: dataset.stats,
  uploaded_at: dataset.uploaded_at,
};

const insightsResponse = {
  dataset_id: dataset.id,
  owner_sub: dataset.owner_sub,
  team_id: dataset.team_id,
  original_filename: dataset.original_filename,
  insights,
  chart_specs: chartSpecs,
  stats: dataset.stats,
  uploaded_at: dataset.uploaded_at,
};

test.beforeEach(async ({ page }) => {
  await mockShellApis(page);
});

test('upload, detection, insights, charts, and drill-down flow', async ({ page }) => {
  await page.route('**/api/datasets/preview', (route) => fulfillJson(route, { preview }));
  await page.route('**/api/datasets/upload', (route) => fulfillJson(route, { dataset }));
  await page.route('**/api/datasets/dataset-1/insights', (route) =>
    fulfillJson(route, insightsResponse),
  );

  await page.goto('/insights');
  await page
    .locator('input[type="file"]')
    .setInputFiles({ name: 'farm-yields.csv', mimeType: 'text/csv', buffer: Buffer.from(csv) });

  await expect(page.getByText('3', { exact: true }).first()).toBeVisible();
  await expect(page.getByRole('columnheader', { name: 'Yield' })).toBeVisible();

  await page.getByRole('button', { name: 'Commit dataset' }).click();

  await expect(
    page.getByRole('heading', { name: 'Dataset is ready for a quick scan' }),
  ).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Yield over Season' })).toBeVisible();
  await expect(
    page.getByRole('heading', { name: 'Field, season, and metric detail' }),
  ).toBeVisible();

  await page.getByLabel('Field column').selectOption({ label: 'Field' });
  await page.getByLabel('Field value').selectOption({ label: 'North' });
  await page.locator('select').nth(3).selectOption({ label: 'Yield' });

  await expect(page.getByText('2 of 3 rows')).toBeVisible();
  await expect(page.getByText('195')).toBeVisible();
});

test('team invite, member roster, and shared dataset view path', async ({ page }) => {
  await mockTeamApis(page);

  await page.goto('/team');
  await page.getByLabel('Team ID').fill('team-1');
  await page.getByLabel('Dataset ID').fill('dataset-1');

  await page.getByRole('button', { name: 'Refresh' }).click();
  await expect(page.getByText('owner@example.com').first()).toBeVisible();
  await expect(page.getByText('analyst@example.com').first()).toBeVisible();

  await page.getByLabel('Invite email').fill('new-member@example.com');
  await page.getByRole('button', { name: 'Invite' }).click();
  await expect(page.getByText(/Invitation queued for new-member@example.com/)).toBeVisible();

  await page.getByRole('button', { name: 'Load shared view' }).click();
  await expect(page.getByRole('heading', { name: 'farm-yields.csv' })).toBeVisible();
  await expect(
    page.getByRole('heading', { name: 'Dataset is ready for a quick scan' }),
  ).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Recommendations' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Column profile' })).toBeVisible();
  await expect(page.getByText('field name')).toBeVisible();
});

async function mockShellApis(page: Page) {
  await page.route('**/api/health', (route) =>
    fulfillJson(route, {
      status: 'ok',
      service: 'farminsight-backend',
      version: 'test',
      database: 'ok',
    }),
  );
  await page.route('**/api/auth/context', (route) =>
    fulfillJson(route, {
      session: {
        user: {
          sub: 'user-owner',
          email: 'owner@example.com',
          name: 'Owner User',
          picture_url: null,
        },
        is_first_seen: false,
        message: 'Welcome back, Owner User',
      },
      team: { requested_team_id: null },
    }),
  );
}

async function mockTeamApis(page: Page) {
  await page.route('**/api/teams/team-1/members', (route) =>
    fulfillJson(route, {
      team_id: 'team-1',
      members: [
        {
          user_sub: 'user-owner',
          email: 'owner@example.com',
          name: 'Owner User',
          picture_url: null,
          role: 'owner',
          joined_at: '2026-07-20T00:00:00Z',
          last_seen_at: '2026-07-21T00:00:00Z',
        },
        {
          user_sub: 'user-analyst',
          email: 'analyst@example.com',
          name: 'Analyst User',
          picture_url: null,
          role: 'member',
          joined_at: '2026-07-20T01:00:00Z',
          last_seen_at: '2026-07-21T00:00:00Z',
        },
      ],
    }),
  );
  await page.route('**/api/teams/team-1/invitations', (route) =>
    fulfillJson(route, {
      invitation_id: 'invitation-1',
      team_id: 'team-1',
      email: 'new-member@example.com',
      expires_at: '2026-07-28T00:00:00Z',
      email_delivery: { status: 'sent', provider_message_id: 'message-1' },
    }),
  );
  await page.route('**/api/teams/team-1/datasets/dataset-1/schema', (route) =>
    fulfillJson(route, schemaResponse),
  );
  await page.route('**/api/teams/team-1/datasets/dataset-1/insights', (route) =>
    fulfillJson(route, insightsResponse),
  );
  await page.route('**/api/teams/team-1/datasets/dataset-1', (route) =>
    fulfillJson(route, { dataset }),
  );
}

async function fulfillJson(route: Route, body: unknown) {
  await route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify(body),
  });
}
