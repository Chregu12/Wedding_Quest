import { Component, inject, signal, computed, OnInit, OnDestroy } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import { Subscription, interval } from 'rxjs';
import { switchMap } from 'rxjs/operators';
import { GameService, RoundInfo, CloseRoundResponse } from '../../services/game.service';
import { ScoreService } from '../../services/score.service';
import { WebSocketService, WsMessage, WsQuestionStarted, WsRoundClosed, WsScoresUpdated } from '../../services/websocket.service';
import { PlayerScore } from '../../models/score.model';

type GamePhase = 'starting' | 'question' | 'round-closed' | 'ich-oder-du' | 'couple-answered' | 'game-over';

@Component({
  selector: 'app-game-control',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './game-control.component.html',
})
export class GameControlComponent implements OnInit, OnDestroy {
  private route = inject(ActivatedRoute);
  private router = inject(Router);
  private gameService = inject(GameService);
  private scoreService = inject(ScoreService);
  private wsService = inject(WebSocketService);

  code = signal('');
  sessionId = signal('');
  phase = signal<GamePhase>('starting');
  currentRound = signal<RoundInfo | null>(null);
  closedRound = signal<CloseRoundResponse | null>(null);
  scores = signal<PlayerScore[]>([]);
  error = signal<string | null>(null);
  loading = signal(false);

  private wsSub?: Subscription;
  private scorePollSub?: Subscription;

  ngOnInit(): void {
    this.code.set(this.route.snapshot.paramMap.get('code') ?? '');

    // Retrieve session UUID stored during session creation for WebSocket
    const storedSessionId = localStorage.getItem(`session_id_${this.code()}`);
    if (storedSessionId) {
      this.sessionId.set(storedSessionId);
      this.connectWebSocket(storedSessionId);
    }

    this.startPollingScores();
    this.startFirstRound();
  }

  ngOnDestroy(): void {
    this.wsSub?.unsubscribe();
    this.scorePollSub?.unsubscribe();
    this.wsService.disconnect();
  }

  private connectWebSocket(sessionId: string): void {
    this.wsService.connect(sessionId);
    this.wsSub = this.wsService.messages().subscribe(msg => this.handleWsMessage(msg));
  }

  private handleWsMessage(msg: WsMessage): void {
    switch (msg.type) {
      case 'Connected': {
        console.log('Admin WS connected');
        break;
      }
      case 'QuestionStarted': {
        const q = msg as WsQuestionStarted;
        this.currentRound.set({
          round_id: q.round_id,
          question_text: q.question_text,
          option_a: q.option_a,
          option_b: q.option_b,
          option_c: q.option_c,
          option_d: q.option_d,
          round_number: q.round_number,
          total_questions: q.total_questions,
        });
        this.phase.set('question');
        break;
      }
      case 'RoundClosed': {
        const rc = msg as WsRoundClosed;
        if (this.closedRound()) {
          this.closedRound.set({ ...this.closedRound()!, correct_answer: rc.correct_answer });
        }
        break;
      }
      case 'ScoresUpdated': {
        const su = msg as WsScoresUpdated;
        // Map WS scores format to PlayerScore
        const mapped: PlayerScore[] = su.scores.map(s => ({
          player_id: s.player_id,
          player_name: s.player_name,
          total_score: s.total_score,
          rounds_played: 0,
          last_round_score: 0,
          rank: s.rank,
        }));
        this.scores.set(mapped);
        break;
      }
      case 'GameEnded': {
        this.phase.set('game-over');
        break;
      }
    }
  }

  private startPollingScores(): void {
    this.scorePollSub = interval(3000).pipe(
      switchMap(() => this.scoreService.getScores(this.code()))
    ).subscribe({
      next: (scores) => this.scores.set(scores),
      error: (err) => console.error('Score polling error', err)
    });
  }

  startFirstRound(): void {
    this.loading.set(true);
    this.gameService.startGame(this.code()).subscribe({
      next: (round) => {
        this.loading.set(false);
        this.currentRound.set(round);
        this.phase.set('question');
      },
      error: (err) => {
        this.loading.set(false);
        // Game might already be started, try to get state
        this.gameService.getState(this.code()).subscribe({
          next: (state) => {
            if (state.status === 'active' || state.status === 'running') {
              this.phase.set('question');
            }
          },
          error: () => this.error.set('Fehler beim Starten des Spiels.')
        });
        console.error(err);
      }
    });
  }

  closeRound(): void {
    this.loading.set(true);
    this.gameService.closeRound(this.code()).subscribe({
      next: (result) => {
        this.loading.set(false);
        this.closedRound.set(result);
        if (result.has_ich_oder_du) {
          this.phase.set('ich-oder-du');
        } else {
          this.phase.set('round-closed');
        }
      },
      error: (err) => {
        this.loading.set(false);
        this.error.set('Fehler beim Beenden der Runde.');
        console.error(err);
      }
    });
  }

  submitCoupleAnswer(answer: 'ich' | 'du'): void {
    this.loading.set(true);
    this.gameService.submitCoupleAnswer(this.code(), { answer }).subscribe({
      next: () => {
        this.loading.set(false);
        this.phase.set('couple-answered');
      },
      error: (err) => {
        this.loading.set(false);
        this.error.set('Fehler beim Senden der Antwort.');
        console.error(err);
      }
    });
  }

  nextQuestion(): void {
    this.loading.set(true);
    this.closedRound.set(null);
    this.gameService.nextQuestion(this.code()).subscribe({
      next: (round) => {
        this.loading.set(false);
        if (round) {
          this.currentRound.set(round);
          this.phase.set('question');
        } else {
          this.phase.set('game-over');
        }
      },
      error: (err) => {
        this.loading.set(false);
        // 204 might be treated as null/empty - check if game ended
        this.phase.set('game-over');
        console.error(err);
      }
    });
  }

  getOptionLabel(option: string): string {
    const map: Record<string, string> = { A: 'A', B: 'B', C: 'C', D: 'D' };
    return map[option] ?? option;
  }

  isCorrectOption(option: string): boolean {
    const closed = this.closedRound();
    return closed !== null && closed.correct_answer === option;
  }

  getOptionClass(option: string): string {
    const closed = this.closedRound();
    if (!closed) {
      const colors: Record<string, string> = {
        A: 'bg-blue-100 border-blue-300 text-blue-800',
        B: 'bg-green-100 border-green-300 text-green-800',
        C: 'bg-orange-100 border-orange-300 text-orange-800',
        D: 'bg-purple-100 border-purple-300 text-purple-800',
      };
      return colors[option] ?? 'bg-gray-100 border-gray-300';
    }
    if (closed.correct_answer === option) {
      return 'bg-green-200 border-green-500 text-green-900 ring-2 ring-green-400';
    }
    return 'bg-gray-100 border-gray-200 text-gray-500 opacity-60';
  }

  goToSetup(): void {
    this.router.navigate(['/admin/sessions', this.code()]);
  }
}
