export type TeamMember = {
  user_sub: string;
  email: string;
  name: string | null;
  picture_url: string | null;
  role: string;
  joined_at: string;
  last_seen_at: string;
};

export type EmailDelivery = {
  status: 'sent' | 'skipped' | 'rate_limited' | 'failed';
  provider_message_id: string | null;
};

export type TeamInvitation = {
  invitation_id: string;
  team_id: string;
  email: string;
  expires_at: string;
  email_delivery: EmailDelivery;
};

type TeamMembersResponse = {
  team_id: string;
  members: TeamMember[];
};

export async function fetchTeamMembers(teamId: string, signal?: AbortSignal) {
  const response = await fetch(`/api/teams/${encodeURIComponent(teamId)}/members`, {
    credentials: 'include',
    signal,
  });

  if (!response.ok) {
    throw new Error(await teamErrorMessage(response, 'Team members fetch failed'));
  }

  return (await response.json()) as TeamMembersResponse;
}

export async function createTeamInvitation(teamId: string, email: string, signal?: AbortSignal) {
  const response = await fetch(`/api/teams/${encodeURIComponent(teamId)}/invitations`, {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email }),
    signal,
  });

  if (!response.ok) {
    throw new Error(await teamErrorMessage(response, 'Team invitation failed'));
  }

  return (await response.json()) as TeamInvitation;
}

async function teamErrorMessage(response: Response, fallback: string) {
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
