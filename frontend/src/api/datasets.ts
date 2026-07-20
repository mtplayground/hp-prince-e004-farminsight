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

export type DatasetRecord = {
  id: string;
  owner_sub: string;
  team_id: string | null;
  original_filename: string;
  storage: StoredFileReference;
  row_count: number | null;
  column_count: number | null;
  column_names: string[];
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

function datasetForm(file: File) {
  const form = new FormData();
  form.append('file', file);
  return form;
}

async function datasetErrorMessage(response: Response, fallback: string) {
  try {
    const body = (await response.json()) as { error?: string };
    if (body.error) {
      return `${fallback}: ${body.error}`;
    }
  } catch {
    // Use the HTTP status when the API cannot return JSON.
  }

  return `${fallback}: ${response.status}`;
}
