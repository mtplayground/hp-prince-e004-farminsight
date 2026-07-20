export type AuthAction = 'login' | 'register';

const defaultReturnTo = '/insights';

export function authRedirectUrl(action: AuthAction, returnTo = defaultReturnTo) {
  const params = new URLSearchParams({ return_to: safeFrontendPath(returnTo) });
  return `/api/auth/${action}?${params.toString()}`;
}

export function startAuthRedirect(action: AuthAction, returnTo = defaultReturnTo) {
  window.location.assign(authRedirectUrl(action, returnTo));
}

function safeFrontendPath(path: string) {
  if (!path.startsWith('/') || path.startsWith('//') || path.startsWith('/api')) {
    return defaultReturnTo;
  }

  return path;
}
