export type AuthAction = 'login' | 'register';

const defaultReturnTo = '/insights';

export type UserIdentity = {
  sub: string;
  email: string;
  name: string | null;
  picture_url: string | null;
};

export type AuthSession = {
  user: UserIdentity;
  is_first_seen: boolean;
  message: string;
};

export type TeamContext = {
  requested_team_id: string | null;
};

export type AuthContextResponse = {
  session: AuthSession;
  team: TeamContext;
};

export function authRedirectUrl(action: AuthAction, returnTo = defaultReturnTo) {
  const params = new URLSearchParams({ return_to: safeFrontendPath(returnTo) });
  return `/api/auth/${action}?${params.toString()}`;
}

export function startAuthRedirect(action: AuthAction, returnTo = defaultReturnTo) {
  window.location.assign(authRedirectUrl(action, returnTo));
}

export async function fetchAuthContext(signal?: AbortSignal) {
  const response = await fetch('/api/auth/context', {
    credentials: 'include',
    headers: { Accept: 'application/json' },
    signal,
  });

  if (response.status === 401) {
    return null;
  }

  if (!response.ok) {
    throw new Error(`Auth context returned ${response.status}`);
  }

  return response.json() as Promise<AuthContextResponse>;
}

function safeFrontendPath(path: string) {
  if (!path.startsWith('/') || path.startsWith('//') || path.startsWith('/api')) {
    return defaultReturnTo;
  }

  return path;
}
