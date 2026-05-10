import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

const ENGINE_API = 'http://localhost:3003';

export interface RoundInfo {
  round_id: string;
  question_text: string;
  option_a: string;
  option_b: string;
  option_c: string;
  option_d: string;
  round_number: number;
  total_questions: number;
}

export interface GameState {
  status: string;
  current_round_id?: string;
  current_round_number: number;
  total_questions: number;
}

export interface AnswerRequest {
  player_id: string;
  player_name: string;
  answer: 'A' | 'B' | 'C' | 'D';
}

export interface AnswerResponse {
  accepted: boolean;
  is_correct: boolean;
}

export interface CloseRoundResponse {
  correct_answer: string;
  has_ich_oder_du: boolean;
  ich_oder_du_text?: string;
}

export interface CoupleAnswerRequest {
  answer: 'ich' | 'du';
}

@Injectable({ providedIn: 'root' })
export class GameService {
  private http = inject(HttpClient);

  startGame(code: string): Observable<RoundInfo> {
    return this.http.post<RoundInfo>(`${ENGINE_API}/games/${code}/start`, {});
  }

  getState(code: string): Observable<GameState> {
    return this.http.get<GameState>(`${ENGINE_API}/games/${code}/state`);
  }

  submitAnswer(code: string, request: AnswerRequest): Observable<AnswerResponse> {
    return this.http.post<AnswerResponse>(`${ENGINE_API}/games/${code}/answer`, request);
  }

  closeRound(code: string): Observable<CloseRoundResponse> {
    return this.http.post<CloseRoundResponse>(`${ENGINE_API}/games/${code}/close-round`, {});
  }

  submitCoupleAnswer(code: string, request: CoupleAnswerRequest): Observable<void> {
    return this.http.post<void>(`${ENGINE_API}/games/${code}/couple-answer`, request);
  }

  nextQuestion(code: string): Observable<RoundInfo | null> {
    return this.http.post<RoundInfo | null>(`${ENGINE_API}/games/${code}/next-question`, {});
  }
}
