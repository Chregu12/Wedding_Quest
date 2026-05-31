import { Component, inject, signal, OnInit } from '@angular/core';
import { Router, RouterLink } from '@angular/router';
import { CommonModule } from '@angular/common';
import { SessionService } from '../../services/session.service';
import { Session } from '../../models/session.model';

@Component({
  selector: 'app-session-list',
  standalone: true,
  imports: [CommonModule, RouterLink],
  templateUrl: './session-list.component.html',
})
export class SessionListComponent implements OnInit {
  private sessionService = inject(SessionService);
  private router = inject(Router);

  sessions = signal<Session[]>([]);
  loading = signal(true);

  ngOnInit(): void {
    this.loadSessions();
  }

  loadSessions(): void {
    this.loading.set(true);
    this.sessionService.list().subscribe({
      next: (sessions) => {
        this.sessions.set(sessions);
        this.loading.set(false);
      },
      error: () => {
        this.loading.set(false);
      }
    });
  }

  openSession(code: string): void {
    this.router.navigate(['/admin/sessions', code]);
  }

  createNew(): void {
    this.router.navigate(['/admin/new']);
  }
}
