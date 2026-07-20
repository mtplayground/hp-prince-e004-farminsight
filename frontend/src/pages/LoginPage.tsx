import { LogIn } from 'lucide-react';

import { authRedirectUrl, startAuthRedirect } from '../api/auth';
import { AuthPanel } from './AuthPanel';
import { useAuthReturnPath } from './useAuthReturnPath';

export function LoginPage() {
  const returnTo = useAuthReturnPath();

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
