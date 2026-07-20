import { useCallback, useEffect, useMemo, useState, type ReactNode } from 'react';

import { fetchAuthContext } from '../api/auth';
import { AuthContext, initialAuthState, type AuthState } from './AuthContext';

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>(initialAuthState);

  const refresh = useCallback(async () => {
    setState({ status: 'loading', context: null, error: null });

    try {
      const context = await fetchAuthContext();
      setState(
        context
          ? { status: 'authenticated', context, error: null }
          : { status: 'unauthenticated', context: null, error: null },
      );
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : 'Unable to load auth context';
      setState({ status: 'error', context: null, error: message });
    }
  }, []);

  useEffect(() => {
    const controller = new AbortController();

    fetchAuthContext(controller.signal)
      .then((context) => {
        setState(
          context
            ? { status: 'authenticated', context, error: null }
            : { status: 'unauthenticated', context: null, error: null },
        );
      })
      .catch((error: unknown) => {
        if (controller.signal.aborted) {
          return;
        }
        const message = error instanceof Error ? error.message : 'Unable to load auth context';
        setState({ status: 'error', context: null, error: message });
      });

    return () => controller.abort();
  }, []);

  const value = useMemo(() => ({ ...state, refresh }), [refresh, state]);

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
