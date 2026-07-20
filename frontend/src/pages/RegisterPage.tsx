import { UserPlus } from 'lucide-react';
import { Navigate } from 'react-router-dom';

import { authRedirectUrl, startAuthRedirect } from '../api/auth';
import { useAuth } from '../auth/useAuth';
import { AuthPanel } from './AuthPanel';
import { useAuthReturnPath } from './useAuthReturnPath';

export function RegisterPage() {
  const auth = useAuth();
  const returnTo = useAuthReturnPath();

  if (auth.status === 'authenticated') {
    return <Navigate to={returnTo} replace />;
  }

  return (
    <AuthPanel
      eyebrow="New account"
      title="Create account"
      description="Start a workspace account and return directly to the Insights view."
      primaryLabel="Create account"
      primaryIcon={UserPlus}
      primaryHref={authRedirectUrl('register', returnTo)}
      onPrimaryClick={() => startAuthRedirect('register', returnTo)}
      secondaryLabel="Sign in"
      secondaryTo="/login"
    />
  );
}
