import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { PlayerScore } from '../models/score.model';

const SCORING_API = 'http://localhost:3004';

@Injectable({ providedIn: 'root' })
export class ScoreService {
  private http = inject(HttpClient);

  getScores(code: string): Observable<PlayerScore[]> {
    return this.http.get<PlayerScore[]>(`${SCORING_API}/scores/${code}`);
  }
}
