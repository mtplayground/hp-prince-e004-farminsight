CREATE EXTENSION IF NOT EXISTS pgcrypto;

DO $$
BEGIN
  CREATE TYPE team_membership_role AS ENUM ('owner', 'member');
EXCEPTION
  WHEN duplicate_object THEN NULL;
END;
$$;

CREATE TABLE IF NOT EXISTS teams (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  created_by_sub TEXT NOT NULL REFERENCES users(sub) ON DELETE RESTRICT,
  shared_dataset_id UUID,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT teams_name_not_blank CHECK (LENGTH(BTRIM(name)) > 0)
);

CREATE TABLE IF NOT EXISTS team_memberships (
  team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
  user_sub TEXT NOT NULL REFERENCES users(sub) ON DELETE CASCADE,
  role team_membership_role NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (team_id, user_sub)
);

CREATE UNIQUE INDEX IF NOT EXISTS team_memberships_one_owner_per_team_idx
ON team_memberships (team_id)
WHERE role = 'owner';

CREATE INDEX IF NOT EXISTS teams_created_by_sub_idx ON teams (created_by_sub);
CREATE INDEX IF NOT EXISTS teams_shared_dataset_id_idx ON teams (shared_dataset_id);
CREATE INDEX IF NOT EXISTS team_memberships_user_sub_idx ON team_memberships (user_sub);
CREATE INDEX IF NOT EXISTS team_memberships_role_idx ON team_memberships (role);

DROP TRIGGER IF EXISTS teams_set_updated_at ON teams;
CREATE TRIGGER teams_set_updated_at
BEFORE UPDATE ON teams
FOR EACH ROW
EXECUTE FUNCTION set_updated_at_timestamp();

DROP TRIGGER IF EXISTS team_memberships_set_updated_at ON team_memberships;
CREATE TRIGGER team_memberships_set_updated_at
BEFORE UPDATE ON team_memberships
FOR EACH ROW
EXECUTE FUNCTION set_updated_at_timestamp();
