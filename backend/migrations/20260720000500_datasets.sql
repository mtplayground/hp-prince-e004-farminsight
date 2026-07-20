CREATE TABLE IF NOT EXISTS datasets (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_sub TEXT NOT NULL REFERENCES users(sub) ON DELETE CASCADE,
  team_id UUID REFERENCES teams(id) ON DELETE SET NULL,
  original_filename TEXT NOT NULL,
  storage_bucket TEXT NOT NULL,
  storage_key TEXT NOT NULL,
  content_type TEXT,
  byte_size BIGINT NOT NULL DEFAULT 0,
  row_count BIGINT,
  column_count INTEGER,
  column_names JSONB NOT NULL DEFAULT '[]'::jsonb,
  stats JSONB NOT NULL DEFAULT '{}'::jsonb,
  uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT datasets_original_filename_not_blank CHECK (LENGTH(BTRIM(original_filename)) > 0),
  CONSTRAINT datasets_storage_bucket_not_blank CHECK (LENGTH(BTRIM(storage_bucket)) > 0),
  CONSTRAINT datasets_storage_key_not_blank CHECK (LENGTH(BTRIM(storage_key)) > 0),
  CONSTRAINT datasets_byte_size_non_negative CHECK (byte_size >= 0),
  CONSTRAINT datasets_row_count_non_negative CHECK (row_count IS NULL OR row_count >= 0),
  CONSTRAINT datasets_column_count_non_negative CHECK (column_count IS NULL OR column_count >= 0),
  CONSTRAINT datasets_column_names_array CHECK (jsonb_typeof(column_names) = 'array'),
  CONSTRAINT datasets_stats_object CHECK (jsonb_typeof(stats) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS datasets_storage_reference_idx
ON datasets (storage_bucket, storage_key);

CREATE INDEX IF NOT EXISTS datasets_owner_uploaded_at_idx ON datasets (owner_sub, uploaded_at DESC);
CREATE INDEX IF NOT EXISTS datasets_team_uploaded_at_idx ON datasets (team_id, uploaded_at DESC);
CREATE INDEX IF NOT EXISTS datasets_uploaded_at_idx ON datasets (uploaded_at DESC);
CREATE INDEX IF NOT EXISTS datasets_stats_gin_idx ON datasets USING GIN (stats);

DO $$
BEGIN
  ALTER TABLE teams
  ADD CONSTRAINT teams_shared_dataset_id_fkey
  FOREIGN KEY (shared_dataset_id) REFERENCES datasets(id) ON DELETE SET NULL;
EXCEPTION
  WHEN duplicate_object THEN NULL;
END;
$$;

DROP TRIGGER IF EXISTS datasets_set_updated_at ON datasets;
CREATE TRIGGER datasets_set_updated_at
BEFORE UPDATE ON datasets
FOR EACH ROW
EXECUTE FUNCTION set_updated_at_timestamp();
