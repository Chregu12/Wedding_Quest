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
  correct_answer: string;
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
  answer_a: string;
  answer_b: string;
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

export interface WsLuckyBoost extends WsMessage {
  type: 'LuckyBoost';
  player_id: string;
  player_name: string;
  multiplier: number;
  session_code: string;
}

@Injectable({ providedIn: 'root' })
export class WebSocketService {
  private ws: WebSocket | null = null;
  private messageSubject = new Subject<WsMessage>();
  private sessionId: string | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private intentionalClose = false;

  connect(sessionId: string): void {
    this.intentionalClose = false;
    this.sessionId = sessionId;
    if (this.ws) {
      this.intentionalClose = true;
      this.ws.close();
    }
    this.doConnect(sessionId);
  }

  private doConnect(sessionId: string): void {
    // Routed through the gateway (nginx) — same host/port as the page, ws/wss auto-selected.
    const wsProto = window.location.protocol === 'https:' ? 'wss' : 'ws';
    this.ws = new WebSocket(`${wsProto}://${window.location.host}/ws/${sessionId}`);
    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data as string) as WsMessage;
        this.messageSubject.next(data);
      } catch {
        // ignore malformed messages
      }
    };
    this.ws.onerror = () => {};
    this.ws.onclose = () => {
      if (!this.intentionalClose && this.sessionId) {
        // Auto-reconnect after 2 seconds
        this.reconnectTimer = setTimeout(() => {
          if (this.sessionId) {
            this.doConnect(this.sessionId);
          }
        }, 2000);
      }
    };
  }

  messages(): Observable<WsMessage> {
    return this.messageSubject.asObservable();
  }

  disconnect(): void {
    this.intentionalClose = true;
    this.sessionId = null;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.ws?.close();
    this.ws = null;
  }
}
