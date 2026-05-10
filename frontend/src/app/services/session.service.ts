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

const SESSION_API = 'http://localhost:3002';

@Injectable({ providedIn: 'root' })
export class SessionService {
  private http = inject(HttpClient);

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
}
