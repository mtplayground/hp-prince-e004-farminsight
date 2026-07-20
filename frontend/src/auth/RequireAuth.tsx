import { Loader2 } from 'lucide-react';
import { Navigate, Outlet, useLocation } from 'react-router-dom';

import { useAuth } from './useAuth';

export function RequireAuth() {
  const auth = useAuth();
  const location = useLocation();

  if (auth.status === 'loading') {
    return (
      <section className="rounded-md border border-stone-200 bg-white p-6">
        <div className="flex items-center gap-3">
          <Loader2 className="h-5 w-5 animate-spin text-field" aria-hidden="true" />
          <div>
            <h2 className="text-lg font-semibold">Opening workspace</h2>
            <p className="mt-1 text-sm text-stone-600">Checking account access.</p>
          </div>
        </div>
      </section>
    );
  }

  if (auth.status !== 'authenticated') {
    const returnTo = `${location.pathname}${location.search}`;
    const loginPath = `/login?return_to=${encodeURIComponent(returnTo)}`;

    return <Navigate to={loginPath} replace state={{ from: location }} />;
  }

  return <Outlet />;
}
