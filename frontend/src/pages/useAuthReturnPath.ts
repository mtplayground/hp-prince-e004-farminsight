import { useLocation } from 'react-router-dom';

export function useAuthReturnPath() {
  const location = useLocation();
  const params = new URLSearchParams(location.search);
  const returnTo = params.get('return_to') ?? '/insights';

  if (!returnTo.startsWith('/') || returnTo.startsWith('//') || returnTo.startsWith('/api')) {
    return '/insights';
  }

  return returnTo;
}
