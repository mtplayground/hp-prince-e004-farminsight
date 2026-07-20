import { LogIn } from 'lucide-react';
import { Navigate } from 'react-router-dom';

import { authRedirectUrl, startAuthRedirect } from '../api/auth';
import { useAuth } from '../auth/useAuth';
import { AuthPanel } from './AuthPanel';
import { useAuthReturnPath } from './useAuthReturnPath';

export function LoginPage() {
  const auth = useAuth();
  const returnTo = useAuthReturnPath();

  if (auth.status === 'authenticated') {
    return <Navigate to={returnTo} replace />;
  }

  return (
    <AuthPanel
      eyebrow="Account access"
      title="Sign in"
      description="Open your Insights workspace and continue working with your datasets."
      primaryLabel="Continue"
      primaryIcon={LogIn}
      primaryHref={authRedirectUrl('login', returnTo)}
      onPrimaryClick={() => startAuthRedirect('login', returnTo)}
      secondaryLabel="Create an account"
      secondaryTo="/register"
    />
  );
}
