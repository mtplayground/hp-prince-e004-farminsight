ALTER TABLE datasets
ADD COLUMN IF NOT EXISTS cached_insights JSONB NOT NULL DEFAULT '[]'::jsonb;

ALTER TABLE datasets
ADD COLUMN IF NOT EXISTS cached_chart_specs JSONB NOT NULL DEFAULT '[]'::jsonb;

DO $$
BEGIN
  ALTER TABLE datasets
  ADD CONSTRAINT datasets_cached_insights_array CHECK (jsonb_typeof(cached_insights) = 'array');
EXCEPTION
  WHEN duplicate_object THEN NULL;
END;
$$;

DO $$
BEGIN
  ALTER TABLE datasets
  ADD CONSTRAINT datasets_cached_chart_specs_array CHECK (jsonb_typeof(cached_chart_specs) = 'array');
EXCEPTION
  WHEN duplicate_object THEN NULL;
END;
$$;

CREATE INDEX IF NOT EXISTS datasets_cached_insights_gin_idx
ON datasets USING GIN (cached_insights);

CREATE INDEX IF NOT EXISTS datasets_cached_chart_specs_gin_idx
ON datasets USING GIN (cached_chart_specs);
