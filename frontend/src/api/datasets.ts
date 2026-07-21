export type CsvPreview = {
  columns: string[];
  rows: string[][];
  row_count: number;
  column_count: number;
  delimiter: string;
  truncated: boolean;
  warnings: string[];
};

export type StoredFileReference = {
  bucket: string;
  key: string;
  content_type: string | null;
  byte_size: number;
};

export type DatasetInsight = {
  kind: string;
  title: string;
  summary: string;
  evidence: string[];
  confidence: number;
};

export type ChartField = {
  index: number;
  name: string;
};

export type DatasetChartSpec = {
  id: string;
  chart_type: string;
  title: string;
  rationale: string;
  x: ChartField;
  y: ChartField;
  series: ChartField | null;
  aggregation: string | null;
  confidence: number;
};

export type DatasetRecord = {
  id: string;
  owner_sub: string;
  team_id: string | null;
  original_filename: string;
  storage: StoredFileReference;
  row_count: number | null;
  column_count: number | null;
  column_names: string[];
  detected_schema: Record<string, unknown>;
  column_stats: Record<string, unknown>[];
  cached_insights: DatasetInsight[];
  cached_chart_specs: DatasetChartSpec[];
  stats: Record<string, unknown>;
  uploaded_at: string;
  created_at: string;
  updated_at: string;
};

type PreviewResponse = {
  preview: CsvPreview;
};

type UploadResponse = {
  dataset: DatasetRecord;
};

type TeamDatasetResponse = {
  dataset: DatasetRecord;
};

export type DatasetSchemaResponse = {
  dataset_id: string;
  owner_sub: string;
  team_id: string | null;
  original_filename: string;
  row_count: number | null;
  column_count: number | null;
  column_names: string[];
  detected_schema: Record<string, unknown>;
  column_stats: Record<string, unknown>[];
  cached_insights: DatasetInsight[];
  cached_chart_specs: DatasetChartSpec[];
  stats: Record<string, unknown>;
  uploaded_at: string;
};

export type DatasetInsightsResponse = {
  dataset_id: string;
  owner_sub: string;
  team_id: string | null;
  original_filename: string;
  insights: DatasetInsight[];
  chart_specs: DatasetChartSpec[];
  stats: Record<string, unknown>;
  uploaded_at: string;
};

export async function previewDatasetFile(file: File, signal?: AbortSignal) {
  const response = await fetch('/api/datasets/preview', {
    method: 'POST',
    credentials: 'include',
    body: datasetForm(file),
    signal,
  });

  if (!response.ok) {
    throw new Error(await datasetErrorMessage(response, 'CSV preview failed'));
  }

  const body = (await response.json()) as PreviewResponse;
  return body.preview;
}

export async function uploadDatasetFile(file: File, signal?: AbortSignal) {
  const response = await fetch('/api/datasets/upload', {
    method: 'POST',
    credentials: 'include',
    body: datasetForm(file),
    signal,
  });

  if (!response.ok) {
    throw new Error(await datasetErrorMessage(response, 'Dataset upload failed'));
  }

  const body = (await response.json()) as UploadResponse;
  return body.dataset;
}

export async function fetchDatasetSchema(datasetId: string, signal?: AbortSignal) {
  const response = await fetch(`/api/datasets/${encodeURIComponent(datasetId)}/schema`, {
    credentials: 'include',
    signal,
  });

  if (!response.ok) {
    throw new Error(await datasetErrorMessage(response, 'Dataset schema fetch failed'));
  }

  return (await response.json()) as DatasetSchemaResponse;
}

export async function fetchDatasetInsights(datasetId: string, signal?: AbortSignal) {
  const response = await fetch(`/api/datasets/${encodeURIComponent(datasetId)}/insights`, {
    credentials: 'include',
    signal,
  });

  if (!response.ok) {
    throw new Error(await datasetErrorMessage(response, 'Dataset insights fetch failed'));
  }

  return (await response.json()) as DatasetInsightsResponse;
}

export async function fetchTeamDataset(teamId: string, datasetId: string, signal?: AbortSignal) {
  const response = await fetch(teamDatasetPath(teamId, datasetId), {
    credentials: 'include',
    signal,
  });

  if (!response.ok) {
    throw new Error(await datasetErrorMessage(response, 'Team dataset fetch failed'));
  }

  const body = (await response.json()) as TeamDatasetResponse;
  return body.dataset;
}

export async function fetchTeamDatasetSchema(
  teamId: string,
  datasetId: string,
  signal?: AbortSignal,
) {
  const response = await fetch(`${teamDatasetPath(teamId, datasetId)}/schema`, {
    credentials: 'include',
    signal,
  });

  if (!response.ok) {
    throw new Error(await datasetErrorMessage(response, 'Team dataset schema fetch failed'));
  }

  return (await response.json()) as DatasetSchemaResponse;
}

export async function fetchTeamDatasetInsights(
  teamId: string,
  datasetId: string,
  signal?: AbortSignal,
) {
  const response = await fetch(`${teamDatasetPath(teamId, datasetId)}/insights`, {
    credentials: 'include',
    signal,
  });

  if (!response.ok) {
    throw new Error(await datasetErrorMessage(response, 'Team dataset insights fetch failed'));
  }

  return (await response.json()) as DatasetInsightsResponse;
}

function datasetForm(file: File) {
  const form = new FormData();
  form.append('file', file);
  return form;
}

function teamDatasetPath(teamId: string, datasetId: string) {
  return `/api/teams/${encodeURIComponent(teamId)}/datasets/${encodeURIComponent(datasetId)}`;
}

async function datasetErrorMessage(response: Response, fallback: string) {
  try {
    const body = (await response.json()) as { error?: string; message?: string };
    if (body.message) {
      return `${fallback}: ${body.message}`;
    }
    if (body.error) {
      return `${fallback}: ${datasetErrorLabel(body.error)}`;
    }
  } catch {
    // Use the HTTP status when the API cannot return JSON.
  }

  return `${fallback}: ${response.status}`;
}

function datasetErrorLabel(code: string) {
  switch (code) {
    case 'missing_file':
      return 'choose a CSV file before continuing';
    case 'invalid_csv':
      return 'use a CSV, text, semicolon, tab, or pipe-delimited file';
    case 'no_data_rows':
      return 'the CSV has headers but no data rows to analyze';
    case 'upload_too_large':
      return 'the CSV is larger than the 50 MB upload limit';
    case 'object_storage_failed':
      return 'the CSV parsed, but storage failed. Try again shortly';
    case 'dataset_create_failed':
      return 'the CSV parsed, but dataset metadata could not be saved';
    default:
      return code.replaceAll('_', ' ');
  }
}
