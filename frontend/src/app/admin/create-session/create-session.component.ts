import { Component, inject, signal } from '@angular/core';
import { Router, RouterLink } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { CommonModule } from '@angular/common';
import { SessionService } from '../../services/session.service';

@Component({
  selector: 'app-create-session',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink],
  templateUrl: './create-session.component.html',
})
export class CreateSessionComponent {
  private router = inject(Router);
  private sessionService = inject(SessionService);

  personAName = signal('');
  personBName = signal('');
  hostName = signal('');
  loading = signal(false);
  error = signal<string | null>(null);

  submit(): void {
    if (!this.personAName() || !this.personBName() || !this.hostName()) {
      this.error.set('Bitte alle Felder ausfüllen.');
      return;
    }
    this.loading.set(true);
    this.error.set(null);

    this.sessionService.create({
      person_a_name: this.personAName(),
      person_b_name: this.personBName(),
      host_name: this.hostName(),
    }).subscribe({
      next: (res) => {
        this.loading.set(false);
        // Store session UUID so game-control can connect via WebSocket
        localStorage.setItem(`session_id_${res.code}`, res.session_id);
        this.router.navigate(['/admin/sessions', res.code]);
      },
      error: (err) => {
        this.loading.set(false);
        this.error.set('Fehler beim Erstellen der Session. Bitte erneut versuchen.');
        console.error(err);
      }
    });
  }
}
