import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { AuthService } from '../services/auth.service';

/** Protects the /admin subtree: redirects to /login when not authenticated. */
export const adminGuard: CanActivateFn = (_route, state) => {
  const auth = inject(AuthService);
  const router = inject(Router);
  if (auth.isAuthed()) return true;
  return router.createUrlTree(['/login'], { queryParams: { returnUrl: state.url } });
};
