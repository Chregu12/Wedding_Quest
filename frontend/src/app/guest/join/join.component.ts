import { Component, inject, signal, OnInit } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { CommonModule } from '@angular/common';
import { SessionService } from '../../services/session.service';

@Component({
  selector: 'app-join',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './join.component.html',
})
export class JoinComponent implements OnInit {
  private router = inject(Router);
  private route = inject(ActivatedRoute);
  private sessionService = inject(SessionService);

  sessionCode = signal('');
  displayName = signal('');
  loading = signal(false);
  error = signal<string | null>(null);

  ngOnInit(): void {
    // Pre-fill code from query param
    const codeParam = this.route.snapshot.queryParamMap.get('code');
    if (codeParam) {
      this.sessionCode.set(codeParam.toUpperCase());
    }

    // Restore display name from localStorage if present
    const savedName = localStorage.getItem('player_display_name');
    if (savedName) {
      this.displayName.set(savedName);
    }
  }

  join(): void {
    const code = this.sessionCode().trim().toUpperCase();
    const name = this.displayName().trim();

    if (!code) {
      this.error.set('Bitte Session-Code eingeben.');
      return;
    }
    if (!name) {
      this.error.set('Bitte deinen Namen eingeben.');
      return;
    }

    this.loading.set(true);
    this.error.set(null);

    this.sessionService.join(code, { display_name: name }).subscribe({
      next: (res) => {
        this.loading.set(false);
        // Persist for page refresh
        localStorage.setItem('player_id', res.player_id);
        localStorage.setItem('session_id', res.session_id);
        localStorage.setItem('player_display_name', name);
        localStorage.setItem('session_code', code);
        localStorage.setItem(`session_id_${code}`, res.session_id);

        this.router.navigate(['/game', code]);
      },
      error: (err) => {
        this.loading.set(false);
        if (err.status === 404) {
          this.error.set('Session nicht gefunden. Bitte Code überprüfen.');
        } else if (err.status === 400) {
          this.error.set('Session läuft nicht oder ist bereits beendet.');
        } else {
          this.error.set('Fehler beim Beitreten. Bitte erneut versuchen.');
        }
        console.error(err);
      }
    });
  }
}
