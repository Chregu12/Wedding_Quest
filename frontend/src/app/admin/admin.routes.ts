import { Routes } from '@angular/router';

export const adminRoutes: Routes = [
  { path: '', redirectTo: 'new', pathMatch: 'full' },
  {
    path: 'new',
    loadComponent: () =>
      import('./create-session/create-session.component').then(m => m.CreateSessionComponent)
  },
  {
    path: 'sessions/:code',
    loadComponent: () =>
      import('./session-detail/session-detail.component').then(m => m.SessionDetailComponent)
  },
  {
    path: 'sessions/:code/game',
    loadComponent: () =>
      import('./game-control/game-control.component').then(m => m.GameControlComponent)
  },
];
