import {
  Component, inject, signal, computed, OnInit, OnDestroy, effect
} from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { CommonModule } from '@angular/common';
import { QRCodeModule } from 'angularx-qrcode';
import { Subscription, interval } from 'rxjs';
import { SessionService } from '../../services/session.service';
import { ScoreService } from '../../services/score.service';
import { WebSocketService, WsMessage, WsQuestionStarted, WsRoundClosed, WsIchOderDuStarted, WsCoupleAnswered, WsScoresUpdated, WsLuckyBoost, WsGameEnded } from '../../services/websocket.service';
import { WakeLockService } from '../../services/wakelock.service';
import { QrBroadcastService } from '../../services/qr-broadcast.service';
import { DisplayBroadcastService } from '../../services/display-broadcast.service';
import { PlayerScore } from '../../models/score.model';

type DisplayPhase = 'lobby' | 'question' | 'round-result' | 'ich-oder-du' | 'couple-answered' | 'game-over';

interface QuestionInfo {
  round_id: string;
  question_text: string;
  option_a: string;
  option_b: string;
  option_c: string;
  option_d: string;
  round_number: number;
  total_questions: number;
}

@Component({
  selector: 'app-leaderboard-display',
  standalone: true,
  imports: [CommonModule, QRCodeModule],
  templateUrl: './leaderboard-display.component.html',
})
export class LeaderboardDisplayComponent implements OnInit, OnDestroy {
  private route = inject(ActivatedRoute);
  private sessionService = inject(SessionService);
  private scoreService = inject(ScoreService);
  private wsService = inject(WebSocketService);
  private wakeLock = inject(WakeLockService);
  qrBroadcast = inject(QrBroadcastService);
  displayBroadcast = inject(DisplayBroadcastService);

  code = signal('');
  sessionId = signal('');

  phase = signal<DisplayPhase>('lobby');
  scores = signal<PlayerScore[]>([]);
  prevScores = signal<Map<string, number>>(new Map());

  currentQuestion = signal<QuestionInfo | null>(null);
  currentQuestionType = signal<string>('guest_quiz');
  personAName = signal('');
  personBName = signal('');
  correctAnswer = signal<string | null>(null);
  ichOderDuText = signal<string | null>(null);
  coupleAnswer = signal<string | null>(null);
  coupleAnswerA = signal<string | null>(null);
  coupleAnswerB = signal<string | null>(null);

  countdownWidth = signal(100);
  roundNumber = signal(0);
  totalQuestions = signal(0);

  luckyBoostVisible = signal(false);
  luckyBoostPlayer = signal('');
  luckyBoostMultiplier = signal(1);

  private countdownInterval?: ReturnType<typeof setInterval>;
  private pollInterval?: ReturnType<typeof setInterval>;
  private wsSub?: Subscription;
  private boostTimer?: ReturnType<typeof setTimeout>;

  currentMultiplier = computed(() => {
    const timeTaken = (100 - this.countdownWidth()) * 10 / 100;
    if (timeTaken <= 3) return 3;
    if (timeTaken <= 6) return 2;
    return 1;
  });

  top3 = computed(() => this.scores().slice(0, 3));
  rest = computed(() => this.scores().slice(3));

  constructor() {
    // React to display broadcast events (game-over from admin)
    effect(() => {
      const overlay = this.displayBroadcast.current();
      if (overlay?.type === 'game-over') {
        this.phase.set('game-over');
      }
    });
  }

  ngOnInit(): void {
    this.wakeLock.acquire();
    const code = this.route.snapshot.paramMap.get('code') ?? '';
    this.code.set(code);

    this.sessionService.getByCode(code).subscribe({
      next: (session: import('../../models/session.model').Session) => {
        this.sessionId.set(session.id);
        this.personAName.set(session.person_a_name);
        this.personBName.set(session.person_b_name);
        this.wsService.connect(code);
        this.wsSub = this.wsService.messages().subscribe(m => this.handleWs(m));
        this.loadScores();
      },
      error: () => console.error('Session not found:', code),
    });

    // Poll scores every 5s as fallback
    this.pollInterval = setInterval(() => this.loadScores(), 5000);
  }

  ngOnDestroy(): void {
    this.wakeLock.release();
    this.wsSub?.unsubscribe();
    this.wsService.disconnect();
    this.stopCountdown();
    clearInterval(this.pollInterval);
    clearTimeout(this.boostTimer);
  }

  private loadScores(): void {
    this.scoreService.getScores(this.code()).subscribe({
      next: (scores) => this.updateScores(scores),
      error: () => {},
    });
  }

  private updateScores(newScores: PlayerScore[]): void {
    const prev = new Map(this.scores().map(s => [s.player_id, s.total_score]));
    this.prevScores.set(prev);
    this.scores.set(newScores);
  }

  wasImproved(playerId: string): boolean {
    const prev = this.prevScores().get(playerId);
    const curr = this.scores().find(s => s.player_id === playerId)?.total_score ?? 0;
    return prev !== undefined && curr > prev;
  }

  private handleWs(msg: WsMessage): void {
    switch (msg.type) {
      case 'QuestionStarted': {
        const q = msg as WsQuestionStarted;
        this.prevScores.set(new Map(this.scores().map(s => [s.player_id, s.total_score])));
        this.currentQuestionType.set((q as any).question_type || 'guest_quiz');
        this.currentQuestion.set({
          round_id: q.round_id,
          question_text: q.question_text,
          option_a: q.option_a,
          option_b: q.option_b,
          option_c: q.option_c,
          option_d: q.option_d,
          round_number: q.round_number,
          total_questions: q.total_questions,
        });
        this.roundNumber.set(q.round_number);
        this.totalQuestions.set(q.total_questions);
        this.correctAnswer.set(null);
        this.coupleAnswer.set(null);
        this.phase.set('question');
        // Sync timer with server's started_at
        const startedAt = (q as any).started_at;
        const elapsed = startedAt ? (Date.now() - new Date(startedAt).getTime()) / 1000 : 0;
        const remaining = Math.max(0, 10 - elapsed);
        this.startCountdown(remaining > 0 ? remaining : 10);
        break;
      }
      case 'RoundClosed': {
        const rc = msg as WsRoundClosed;
        this.correctAnswer.set(rc.correct_answer);
        this.stopCountdown();
        this.phase.set('round-result');
        break;
      }
      case 'IchOderDuStarted': {
        const iod = msg as WsIchOderDuStarted;
        this.ichOderDuText.set(iod.ich_oder_du_text);
        this.phase.set('ich-oder-du');
        break;
      }
      case 'CoupleAnswered': {
        const ca = msg as WsCoupleAnswered;
        this.coupleAnswer.set(ca.couple_answer);
        this.coupleAnswerA.set(ca.answer_a);
        this.coupleAnswerB.set(ca.answer_b);
        // Don't change phase - wait for RoundClosed (admin clicks "Auflösen")
        break;
      }
      case 'ScoresUpdated': {
        const su = msg as WsScoresUpdated;
        const mapped: PlayerScore[] = su.scores.map(s => ({
          player_id: s.player_id,
          player_name: s.player_name,
          total_score: s.total_score,
          rounds_played: 0,
          last_round_score: 0,
          rank: s.rank,
        }));
        this.updateScores(mapped);
        break;
      }
      case 'LuckyBoost': {
        const lb = msg as WsLuckyBoost;
        this.luckyBoostPlayer.set(lb.player_name);
        this.luckyBoostMultiplier.set(lb.multiplier);
        this.luckyBoostVisible.set(true);
        clearTimeout(this.boostTimer);
        this.boostTimer = setTimeout(() => this.luckyBoostVisible.set(false), 4500);
        break;
      }
      case 'GameEnded': {
        this.stopCountdown();
        this.phase.set('game-over');
        this.loadScores();
        break;
      }
    }
  }

  private startCountdown(seconds: number): void {
    this.stopCountdown();
    this.countdownWidth.set(100);
    const step = 100 / (seconds * 10);
    this.countdownInterval = setInterval(() => {
      const curr = this.countdownWidth();
      if (curr <= 0) { this.stopCountdown(); return; }
      this.countdownWidth.set(Math.max(0, curr - step));
    }, 100);
  }

  private stopCountdown(): void {
    if (this.countdownInterval) {
      clearInterval(this.countdownInterval);
      this.countdownInterval = undefined;
    }
  }

  getAnswerClass(option: string): string {
    const correct = this.correctAnswer();
    if (!correct) return '';
    if (option === correct) return 'bg-green-500 border-green-400 text-white correct-pop';
    return 'opacity-30';
  }

  medalEmoji(rank: number): string {
    return rank === 1 ? '🥇' : rank === 2 ? '🥈' : '🥉';
  }

  podiumHeight(rank: number): string {
    return rank === 1 ? 'h-36' : rank === 2 ? 'h-24' : 'h-16';
  }

  /** Returns the actual rank (1-based) for a player, accounting for ties */
  actualRank(index: number): number {
    const t3 = this.top3();
    if (index === 0) return 1;
    if (t3[index].total_score === t3[0].total_score) return 1;
    if (index === 1) return 2;
    if (t3[index].total_score === t3[1].total_score) return 2;
    return 3;
  }
}
