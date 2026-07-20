import { createContext } from 'react';

import type { AuthContextResponse } from '../api/auth';

export type AuthState =
  | { status: 'loading'; context: null; error: null }
  | { status: 'authenticated'; context: AuthContextResponse; error: null }
  | { status: 'unauthenticated'; context: null; error: null }
  | { status: 'error'; context: null; error: string };

export type AuthContextValue = AuthState & {
  refresh: () => Promise<void>;
};

export const initialAuthState: AuthState = {
  status: 'loading',
  context: null,
  error: null,
};

export const AuthContext = createContext<AuthContextValue | null>(null);
