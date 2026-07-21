import {
  AlertTriangle,
  CheckCircle2,
  FileSpreadsheet,
  Loader2,
  Table2,
  UploadCloud,
  X,
} from 'lucide-react';
import { useCallback, useRef, useState } from 'react';

import {
  type CsvPreview,
  type DatasetRecord,
  previewDatasetFile,
  uploadDatasetFile,
} from '../api/datasets';

type UploadState = 'idle' | 'previewing' | 'ready' | 'uploading' | 'uploaded';

type DatasetUploadPanelProps = {
  onDatasetUploaded: (dataset: DatasetRecord, preview: CsvPreview) => void;
};

export function DatasetUploadPanel({ onDatasetUploaded }: DatasetUploadPanelProps) {
  const inputRef = useRef<HTMLInputElement | null>(null);
  const previewControllerRef = useRef<AbortController | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [file, setFile] = useState<File | null>(null);
  const [preview, setPreview] = useState<CsvPreview | null>(null);
  const [status, setStatus] = useState<UploadState>('idle');
  const [error, setError] = useState<string | null>(null);

  const reset = useCallback(() => {
    previewControllerRef.current?.abort();
    previewControllerRef.current = null;
    setFile(null);
    setPreview(null);
    setStatus('idle');
    setError(null);
    if (inputRef.current) {
      inputRef.current.value = '';
    }
  }, []);

  const selectFile = useCallback((nextFile: File) => {
    previewControllerRef.current?.abort();
    const controller = new AbortController();
    previewControllerRef.current = controller;

    setFile(nextFile);
    setPreview(null);
    setError(null);
    setStatus('previewing');

    previewDatasetFile(nextFile, controller.signal)
      .then((nextPreview) => {
        setPreview(nextPreview);
        setStatus('ready');
      })
      .catch((cause: unknown) => {
        if (controller.signal.aborted) {
          return;
        }
        setStatus('idle');
        setError(cause instanceof Error ? cause.message : 'CSV preview failed');
      });
  }, []);

  const commitUpload = useCallback(() => {
    if (!file || !preview || status !== 'ready') {
      return;
    }
    if (preview.row_count === 0) {
      setError('Add at least one data row before committing this CSV.');
      return;
    }

    const controller = new AbortController();
    setStatus('uploading');
    setError(null);

    uploadDatasetFile(file, controller.signal)
      .then((dataset) => {
        setStatus('uploaded');
        onDatasetUploaded(dataset, preview);
      })
      .catch((cause: unknown) => {
        setStatus('ready');
        setError(cause instanceof Error ? cause.message : 'Dataset upload failed');
      });
  }, [file, onDatasetUploaded, preview, status]);

  const canCommit = status === 'ready' && preview !== null && preview.row_count > 0;

  return (
    <section className="rounded-md border border-stone-200 bg-white">
      <div className="grid gap-0 lg:grid-cols-[360px_1fr]">
        <div className="border-b border-stone-200 p-5 lg:border-b-0 lg:border-r">
          <div className="flex items-start gap-3">
            <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-field text-white">
              <UploadCloud className="h-5 w-5" aria-hidden="true" />
            </span>
            <div>
              <h3 className="text-lg font-semibold">Dataset upload</h3>
              <p className="mt-1 text-sm leading-6 text-stone-600">
                CSV, semicolon, tab, or pipe-delimited files. Uneven rows are padded for preview.
              </p>
            </div>
          </div>

          <button
            type="button"
            onClick={() => inputRef.current?.click()}
            onDragEnter={(event) => {
              event.preventDefault();
              setIsDragging(true);
            }}
            onDragOver={(event) => {
              event.preventDefault();
              setIsDragging(true);
            }}
            onDragLeave={(event) => {
              event.preventDefault();
              setIsDragging(false);
            }}
            onDrop={(event) => {
              event.preventDefault();
              setIsDragging(false);
              const droppedFile = event.dataTransfer.files.item(0);
              if (droppedFile) {
                selectFile(droppedFile);
              }
            }}
            className={[
              'mt-5 flex min-h-44 w-full flex-col items-center justify-center rounded-md border border-dashed px-4 text-center transition',
              isDragging
                ? 'border-field bg-green-50 text-field'
                : 'border-stone-300 bg-stone-50 text-stone-700 hover:border-field hover:bg-green-50',
            ].join(' ')}
          >
            <FileSpreadsheet className="h-8 w-8" aria-hidden="true" />
            <span className="mt-3 text-sm font-semibold">
              {file ? file.name : 'Drop CSV or choose file'}
            </span>
            <span className="mt-1 text-xs text-stone-500">
              {file ? formatBytes(file.size) : 'Maximum upload size: 50 MB'}
            </span>
          </button>

          <input
            ref={inputRef}
            type="file"
            accept=".csv,text/csv,text/plain"
            className="sr-only"
            onChange={(event) => {
              const nextFile = event.currentTarget.files?.item(0);
              if (nextFile) {
                selectFile(nextFile);
              }
            }}
          />

          <div className="mt-4 grid gap-2 sm:grid-cols-2 lg:grid-cols-1">
            <button
              type="button"
              onClick={commitUpload}
              disabled={!canCommit}
              className="inline-flex min-h-10 items-center justify-center gap-2 rounded-md bg-field px-3 text-sm font-semibold text-white disabled:cursor-not-allowed disabled:bg-stone-300 disabled:text-stone-600"
            >
              {status === 'uploading' ? (
                <Loader2 className="h-4 w-4 animate-spin" aria-hidden="true" />
              ) : (
                <CheckCircle2 className="h-4 w-4" aria-hidden="true" />
              )}
              Commit dataset
            </button>
            <button
              type="button"
              onClick={reset}
              disabled={status === 'idle'}
              className="inline-flex min-h-10 items-center justify-center gap-2 rounded-md border border-stone-200 px-3 text-sm font-semibold text-stone-700 hover:bg-stone-100 disabled:cursor-not-allowed disabled:text-stone-400"
            >
              <X className="h-4 w-4" aria-hidden="true" />
              Clear
            </button>
          </div>

          {error && (
            <div className="mt-4 flex gap-2 rounded-md border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
              <p>{error}</p>
            </div>
          )}

          {status === 'uploaded' && (
            <div className="mt-4 flex gap-2 rounded-md border border-green-200 bg-green-50 p-3 text-sm text-green-800">
              <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
              <p>Dataset committed.</p>
            </div>
          )}
        </div>

        <div className="min-w-0 p-5">
          <PreviewSurface preview={preview} status={status} />
        </div>
      </div>
    </section>
  );
}

function PreviewSurface({ preview, status }: { preview: CsvPreview | null; status: UploadState }) {
  if (status === 'previewing') {
    return (
      <div className="flex min-h-72 items-center justify-center rounded-md border border-stone-200 bg-stone-50 text-stone-600">
        <Loader2 className="mr-2 h-5 w-5 animate-spin" aria-hidden="true" />
        Parsing preview
      </div>
    );
  }

  if (!preview) {
    return (
      <div className="flex min-h-72 flex-col items-center justify-center rounded-md border border-dashed border-stone-300 bg-stone-50 px-4 text-center text-stone-600">
        <Table2 className="h-8 w-8 text-field" aria-hidden="true" />
        <p className="mt-3 text-sm font-semibold">Preview appears here</p>
        <p className="mt-1 max-w-sm text-xs leading-5">
          Parsed columns, row samples, and recovery notes load before storage commit.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="grid gap-3 sm:grid-cols-3">
        <PreviewMetric label="Rows" value={preview.row_count.toLocaleString()} />
        <PreviewMetric label="Columns" value={preview.column_count.toLocaleString()} />
        <PreviewMetric label="Delimiter" value={delimiterLabel(preview.delimiter)} />
      </div>

      {preview.warnings.length > 0 && (
        <div className="rounded-md border border-amber-200 bg-amber-50 p-3">
          <div className="flex items-center gap-2 text-sm font-semibold text-amber-900">
            <AlertTriangle className="h-4 w-4" aria-hidden="true" />
            Parser notes
          </div>
          <ul className="mt-2 space-y-1 text-sm text-amber-800">
            {preview.warnings.map((warning) => (
              <li key={warning}>{warning}</li>
            ))}
          </ul>
        </div>
      )}

      <div className="overflow-hidden rounded-md border border-stone-200">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-stone-200 text-left text-sm">
            <thead className="bg-stone-50">
              <tr>
                {preview.columns.map((column) => (
                  <th
                    key={column}
                    scope="col"
                    className="whitespace-nowrap px-3 py-2 font-semibold text-stone-700"
                  >
                    {column}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-100 bg-white">
              {preview.rows.map((row, rowIndex) => (
                <tr key={`${rowIndex}-${row.join('|')}`}>
                  {preview.columns.map((column, columnIndex) => (
                    <td
                      key={`${column}-${columnIndex}`}
                      className="max-w-56 truncate px-3 py-2 text-stone-700"
                    >
                      {row[columnIndex] || <span className="text-stone-400">blank</span>}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {preview.truncated && (
        <p className="text-xs text-stone-500">Showing the first {preview.rows.length} rows.</p>
      )}

      {preview.row_count === 0 && (
        <p className="rounded-md border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800">
          This file has headers but no data rows, so it cannot be committed for insights yet.
        </p>
      )}
    </div>
  );
}

function PreviewMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-stone-200 bg-stone-50 p-3">
      <p className="text-xs font-medium text-stone-500">{label}</p>
      <p className="mt-1 text-lg font-semibold">{value}</p>
    </div>
  );
}

function delimiterLabel(delimiter: string) {
  if (delimiter === '\t') {
    return 'Tab';
  }

  return delimiter;
}

function formatBytes(size: number) {
  if (size < 1024) {
    return `${size} B`;
  }

  if (size < 1024 * 1024) {
    return `${(size / 1024).toFixed(1)} KB`;
  }

  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}
