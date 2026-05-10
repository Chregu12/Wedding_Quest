import { Routes } from '@angular/router';

export const guestRoutes: Routes = [
  {
    path: 'join',
    loadComponent: () =>
      import('./join/join.component').then(m => m.JoinComponent)
  },
  {
    path: 'game/:code',
    loadComponent: () =>
      import('./game/game.component').then(m => m.GuestGameComponent)
  },
];
