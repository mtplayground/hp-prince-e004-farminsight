CREATE TABLE IF NOT EXISTS team_invitations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
  email TEXT NOT NULL,
  token_hash TEXT NOT NULL UNIQUE,
  invited_by_sub TEXT NOT NULL REFERENCES users(sub) ON DELETE RESTRICT,
  accepted_by_sub TEXT REFERENCES users(sub) ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ NOT NULL,
  accepted_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ,
  CONSTRAINT team_invitations_email_not_blank CHECK (LENGTH(BTRIM(email)) > 0),
  CONSTRAINT team_invitations_expires_after_created CHECK (expires_at > created_at)
);

CREATE INDEX IF NOT EXISTS team_invitations_team_id_idx ON team_invitations (team_id);
CREATE INDEX IF NOT EXISTS team_invitations_email_lower_idx ON team_invitations (LOWER(email));
CREATE INDEX IF NOT EXISTS team_invitations_expires_at_idx ON team_invitations (expires_at);
CREATE INDEX IF NOT EXISTS team_invitations_pending_team_email_idx
ON team_invitations (team_id, LOWER(email))
WHERE accepted_at IS NULL AND revoked_at IS NULL;

DROP TRIGGER IF EXISTS team_invitations_set_updated_at ON team_invitations;
CREATE TRIGGER team_invitations_set_updated_at
BEFORE UPDATE ON team_invitations
FOR EACH ROW
EXECUTE FUNCTION set_updated_at_timestamp();
