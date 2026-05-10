import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: '', redirectTo: '/join', pathMatch: 'full' },
  {
    path: 'admin',
    loadChildren: () => import('./admin/admin.routes').then(m => m.adminRoutes)
  },
  {
    path: 'display',
    loadChildren: () => import('./display/display.routes').then(m => m.displayRoutes)
  },
  {
    path: '',
    loadChildren: () => import('./guest/guest.routes').then(m => m.guestRoutes)
  }
];
