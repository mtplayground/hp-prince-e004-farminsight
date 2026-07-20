ALTER TABLE datasets
ADD COLUMN IF NOT EXISTS detected_schema JSONB NOT NULL DEFAULT '{"columns":[]}'::jsonb;

ALTER TABLE datasets
ADD COLUMN IF NOT EXISTS column_stats JSONB NOT NULL DEFAULT '[]'::jsonb;

DO $$
BEGIN
  ALTER TABLE datasets
  ADD CONSTRAINT datasets_detected_schema_object CHECK (jsonb_typeof(detected_schema) = 'object');
EXCEPTION
  WHEN duplicate_object THEN NULL;
END;
$$;

DO $$
BEGIN
  ALTER TABLE datasets
  ADD CONSTRAINT datasets_column_stats_array CHECK (jsonb_typeof(column_stats) = 'array');
EXCEPTION
  WHEN duplicate_object THEN NULL;
END;
$$;

CREATE INDEX IF NOT EXISTS datasets_detected_schema_gin_idx
ON datasets USING GIN (detected_schema);

CREATE INDEX IF NOT EXISTS datasets_column_stats_gin_idx
ON datasets USING GIN (column_stats);
