import { Routes } from '@angular/router';
import { adminGuard } from './guards/admin.guard';

export const routes: Routes = [
  { path: '', redirectTo: '/join', pathMatch: 'full' },
  {
    path: 'login',
    loadComponent: () => import('./login/login.component').then(m => m.LoginComponent)
  },
  {
    path: 'admin',
    canActivate: [adminGuard],
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
