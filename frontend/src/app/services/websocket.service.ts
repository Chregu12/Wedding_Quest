import { Injectable } from '@angular/core';
import { Subject, Observable } from 'rxjs';

export interface WsMessage {
  type: string;
  [key: string]: unknown;
}

export interface WsConnected extends WsMessage {
  type: 'Connected';
  session_code: string;
}

export interface WsQuestionStarted extends WsMessage {
  type: 'QuestionStarted';
  round_id: string;
  question_text: string;
  option_a: string;
  option_b: string;
  option_c: string;
  option_d: string;
  round_number: number;
  total_questions: number;
}

export interface WsRoundClosed extends WsMessage {
  type: 'RoundClosed';
  round_id: string;
  correct_answer: string;
}

export interface WsIchOderDuStarted extends WsMessage {
  type: 'IchOderDuStarted';
  round_id: string;
  ich_oder_du_text: string;
}

export interface WsCoupleAnswered extends WsMessage {
  type: 'CoupleAnswered';
  round_id: string;
  couple_answer: string;
}

export interface WsScoresUpdated extends WsMessage {
  type: 'ScoresUpdated';
  scores: Array<{
    player_id: string;
    player_name: string;
    total_score: number;
    rank: number;
  }>;
}

export interface WsGameEnded extends WsMessage {
  type: 'GameEnded';
  session_code: string;
}

@Injectable({ providedIn: 'root' })
export class WebSocketService {
  private ws: WebSocket | null = null;
  private messageSubject = new Subject<WsMessage>();

  connect(sessionId: string): void {
    if (this.ws) this.disconnect();
    this.ws = new WebSocket(`ws://localhost:3006/ws/${sessionId}`);
    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data as string) as WsMessage;
        this.messageSubject.next(data);
      } catch {
        // ignore malformed messages
      }
    };
    this.ws.onerror = (err) => console.error('WS error', err);
    this.ws.onclose = () => console.log('WS closed');
  }

  messages(): Observable<WsMessage> {
    return this.messageSubject.asObservable();
  }

  disconnect(): void {
    this.ws?.close();
    this.ws = null;
  }
}
