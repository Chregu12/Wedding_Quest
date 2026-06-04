import { Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router, ActivatedRoute } from '@angular/router';
import { AuthService } from '../services/auth.service';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [FormsModule],
  template: `
    <div class="min-h-screen flex items-center justify-center bg-gray-50 p-4">
      <div class="w-full max-w-sm bg-white rounded-2xl shadow-xl p-8">
        <div class="text-center mb-6">
          <div class="text-4xl mb-2">💍🔒</div>
          <h1 class="text-xl font-semibold text-gray-800">Admin-Login</h1>
          <p class="text-sm text-gray-500 mt-1">Geschützter Bereich – Fragen &amp; Spielsteuerung</p>
        </div>

        <form (ngSubmit)="submit()">
          <label class="block text-sm font-medium text-gray-700 mb-1" for="pw">Passwort</label>
          <input
            id="pw"
            type="password"
            autocomplete="current-password"
            [ngModel]="password()"
            (ngModelChange)="password.set($event)"
            name="password"
            [disabled]="loading()"
            class="w-full px-4 py-3 rounded-lg border border-gray-300 focus:border-blue-500 focus:ring-2 focus:ring-blue-200 outline-none mb-3"
            placeholder="••••••••" />

          @if (error()) {
            <p class="text-sm text-red-600 mb-3">Falsches Passwort.</p>
          }

          <button
            type="submit"
            [disabled]="loading() || !password()"
            class="w-full py-3 rounded-lg bg-blue-600 text-white font-medium hover:bg-blue-700 disabled:opacity-50 transition">
            {{ loading() ? 'Prüfe…' : 'Anmelden' }}
          </button>
        </form>
      </div>
    </div>
  `,
})
export class LoginComponent {
  private auth = inject(AuthService);
  private router = inject(Router);
  private route = inject(ActivatedRoute);

  password = signal('');
  error = signal(false);
  loading = signal(false);

  submit(): void {
    if (this.loading() || !this.password()) return;
    this.loading.set(true);
    this.error.set(false);
    this.auth.login(this.password()).subscribe((ok) => {
      this.loading.set(false);
      if (ok) {
        const ret = this.route.snapshot.queryParamMap.get('returnUrl') || '/admin';
        this.router.navigateByUrl(ret);
      } else {
        this.error.set(true);
        this.password.set('');
      }
    });
  }
}
