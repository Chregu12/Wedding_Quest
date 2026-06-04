import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, map } from 'rxjs';
import { PlayerScore } from '../models/score.model';

// Routed through the gateway (nginx) — relative path keeps it single-origin/single-port.
const SCORING_API = '/api/scoring';

@Injectable({ providedIn: 'root' })
export class ScoreService {
  private http = inject(HttpClient);

  getScores(code: string): Observable<PlayerScore[]> {
    return this.http.get<{ session_code: string; scores: PlayerScore[] }>(`${SCORING_API}/scores/${code}`).pipe(
      map(response => response.scores)
    );
  }

  resetScores(code: string): Observable<void> {
    return this.http.delete<void>(`${SCORING_API}/scores/${code}`);
  }
}
