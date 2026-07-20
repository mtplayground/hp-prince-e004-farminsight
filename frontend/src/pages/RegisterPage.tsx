import { UserPlus } from 'lucide-react';

import { authRedirectUrl, startAuthRedirect } from '../api/auth';
import { AuthPanel } from './AuthPanel';
import { useAuthReturnPath } from './useAuthReturnPath';

export function RegisterPage() {
  const returnTo = useAuthReturnPath();

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
