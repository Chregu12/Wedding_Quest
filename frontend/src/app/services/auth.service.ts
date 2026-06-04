import { Injectable, inject, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, map, catchError, of } from 'rxjs';

const FLAG_KEY = 'wq_admin_authed';
const COOKIE = 'wq_admin';

/**
 * Admin authentication for the protected /admin area.
 *
 * The actual gate lives in the nginx gateway: it only serves the admin API
 * endpoints when the `wq_admin` cookie equals the server-side ADMIN_PASSWORD.
 * Here we set that cookie and verify it against the gateway's /admin-check,
 * then remember success in localStorage so the route guard can react instantly.
 */
@Injectable({ providedIn: 'root' })
export class AuthService {
  private http = inject(HttpClient);
  readonly isAuthed = signal<boolean>(localStorage.getItem(FLAG_KEY) === '1');

  /** Set the admin cookie and verify it. Emits true on success. */
  login(password: string): Observable<boolean> {
    document.cookie = `${COOKIE}=${password}; path=/; max-age=604800; samesite=lax`;
    return this.http.get('/admin-check', { observe: 'response' }).pipe(
      map(() => { this.setAuthed(true); return true; }),
      catchError(() => { this.clearCookie(); this.setAuthed(false); return of(false); }),
    );
  }

  logout(): void {
    this.clearCookie();
    this.setAuthed(false);
  }

  private setAuthed(v: boolean): void {
    this.isAuthed.set(v);
    if (v) localStorage.setItem(FLAG_KEY, '1');
    else localStorage.removeItem(FLAG_KEY);
  }

  private clearCookie(): void {
    document.cookie = `${COOKIE}=; path=/; max-age=0`;
  }
}
