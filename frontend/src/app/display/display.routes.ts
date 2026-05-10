import { Routes } from '@angular/router';

export const displayRoutes: Routes = [
  {
    path: ':code',
    loadComponent: () =>
      import('./leaderboard-display/leaderboard-display.component').then(
        (m) => m.LeaderboardDisplayComponent
      ),
  },
];
