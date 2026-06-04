import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import {
  Session,
  CreateSessionRequest,
  CreateSessionResponse,
  JoinSessionRequest,
  JoinSessionResponse,
  PlayersResponse
} from '../models/session.model';

// Routed through the gateway (nginx) — relative path keeps it single-origin/single-port.
const SESSION_API = '/api/session';

@Injectable({ providedIn: 'root' })
export class SessionService {
  private http = inject(HttpClient);

  list(): Observable<Session[]> {
    return this.http.get<Session[]>(`${SESSION_API}/sessions`);
  }

  create(request: CreateSessionRequest): Observable<CreateSessionResponse> {
    return this.http.post<CreateSessionResponse>(`${SESSION_API}/sessions`, request);
  }

  getByCode(code: string): Observable<Session> {
    return this.http.get<Session>(`${SESSION_API}/sessions/${code}`);
  }

  start(code: string): Observable<void> {
    return this.http.post<void>(`${SESSION_API}/sessions/${code}/start`, {});
  }

  join(code: string, request: JoinSessionRequest): Observable<JoinSessionResponse> {
    return this.http.post<JoinSessionResponse>(`${SESSION_API}/sessions/${code}/join`, request);
  }

  getPlayers(code: string): Observable<PlayersResponse> {
    return this.http.get<PlayersResponse>(`${SESSION_API}/sessions/${code}/players`);
  }

  clearPlayers(code: string): Observable<void> {
    return this.http.delete<void>(`${SESSION_API}/sessions/${code}/players`);
  }

  updateSession(code: string, data: { person_a_name: string; person_b_name: string; host_name: string }): Observable<void> {
    return this.http.put<void>(`${SESSION_API}/sessions/${code}`, data);
  }
}
