CREATE TABLE IF NOT EXISTS users (
  sub TEXT PRIMARY KEY,
  email TEXT NOT NULL,
  name TEXT,
  picture_url TEXT,
  credentials_hash TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT users_sub_not_blank CHECK (LENGTH(BTRIM(sub)) > 0),
  CONSTRAINT users_email_not_blank CHECK (LENGTH(BTRIM(email)) > 0)
);

CREATE INDEX IF NOT EXISTS users_email_lower_idx ON users (LOWER(email));
CREATE INDEX IF NOT EXISTS users_last_seen_at_idx ON users (last_seen_at DESC);

CREATE OR REPLACE FUNCTION set_updated_at_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS users_set_updated_at ON users;
CREATE TRIGGER users_set_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION set_updated_at_timestamp();
