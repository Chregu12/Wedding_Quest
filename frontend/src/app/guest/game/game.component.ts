import { Component, inject, signal, computed, OnInit, OnDestroy } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { CommonModule } from '@angular/common';
import { Subscription, interval } from 'rxjs';
import { switchMap } from 'rxjs/operators';
import { GameService, AnswerRequest } from '../../services/game.service';
import { SessionService } from '../../services/session.service';
import { ScoreService } from '../../services/score.service';
import { WakeLockService } from '../../services/wakelock.service';
import { WebSocketService, WsMessage, WsQuestionStarted, WsRoundClosed, WsIchOderDuStarted, WsCoupleAnswered, WsScoresUpdated, WsGameEnded, WsLuckyBoost } from '../../services/websocket.service';
import { PlayerScore } from '../../models/score.model';

type GuestPhase =
  | 'waiting'
  | 'question'
  | 'answered'
  | 'round-result'
  | 'ich-oder-du'
  | 'couple-answered'
  | 'game-over';

interface CurrentQuestion {
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
  selector: 'app-guest-game',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './game.component.html',
})
export class GuestGameComponent implements OnInit, OnDestroy {
  private route = inject(ActivatedRoute);
  private gameService = inject(GameService);
  private sessionService = inject(SessionService);
  private scoreService = inject(ScoreService);
  private wsService = inject(WebSocketService);
  private wakeLock = inject(WakeLockService);

  code = signal('');
  playerId = signal('');
  playerName = signal('');
  sessionId = signal('');
  personAName = signal('');
  personBName = signal('');

  phase = signal<GuestPhase>('waiting');
  currentQuestion = signal<CurrentQuestion | null>(null);
  currentQuestionType = signal<string>('guest_quiz');
  selectedAnswer = signal<string | null>(null);
  correctAnswer = signal<string | null>(null);
  ichOderDuText = signal<string | null>(null);
  coupleAnswer = signal<string | null>(null);
  iodGuess = signal<string | null>(null);
  scores = signal<PlayerScore[]>([]);
  answerSubmitting = signal(false);
  answerSubmitted = signal(false);
  screenActivated = signal(false);

  luckyBoostVisible = signal(false);
  luckyBoostMultiplier = signal(1);

  countdownWidth = signal(100);
  private countdownInterval?: ReturnType<typeof setInterval>;
  private wsSub?: Subscription;
  private pollSub?: Subscription;
  private lastSeenRoundId = '';
  private boostTimer?: ReturnType<typeof setTimeout>;

  currentMultiplier = computed(() => {
    const timeTaken = (100 - this.countdownWidth()) * 30 / 100;
    if (timeTaken <= 10) return 3;
    if (timeTaken <= 20) return 2;
    return 1;
  });

  myRank = computed(() => {
    const id = this.playerId();
    const found = this.scores().find(s => s.player_id === id);
    return found?.rank ?? null;
  });

  myScore = computed(() => {
    const id = this.playerId();
    const found = this.scores().find(s => s.player_id === id);
    return found?.total_score ?? 0;
  });

  ngOnInit(): void {
    this.wakeLock.acquire();
    this.code.set(this.route.snapshot.paramMap.get('code') ?? '');
    this.playerId.set(localStorage.getItem('player_id') ?? '');
    this.playerName.set(localStorage.getItem('player_display_name') ?? '');
    this.sessionId.set(localStorage.getItem('session_id') ?? '');

    // Load couple names for Ich-oder-Du display
    this.sessionService.getByCode(this.code()).subscribe({
      next: (s) => {
        this.personAName.set(s.person_a_name);
        this.personBName.set(s.person_b_name);
      },
      error: () => {}
    });

    if (this.code()) {
      // Connect WS for all game events
      this.wsService.connect(this.code());
      this.wsSub = this.wsService.messages().subscribe(msg => this.handleWsMessage(msg));

      // Start polling immediately - no initial state snapshot needed
      // WS events and polling both use lastSeenRoundId to avoid duplicates
      this.startPolling();
    }
  }

  private startPolling(): void {
    this.pollSub = interval(1500).pipe(
      switchMap(() => this.gameService.getState(this.code()))
    ).subscribe({
      next: (state) => {
        if (!state.current_round_id || !state.question_text) return;
        if (state.current_round_id === this.lastSeenRoundId) return;

        const p = this.phase();
        if (p === 'question' || p === 'answered') return;

        if (state.status === 'question') {
          this.lastSeenRoundId = state.current_round_id;
          this.currentQuestion.set({
            round_id: state.current_round_id,
            question_text: state.question_text,
            option_a: state.option_a || '',
            option_b: state.option_b || '',
            option_c: state.option_c || '',
            option_d: state.option_d || '',
            round_number: state.current_round_number,
            total_questions: state.total_questions,
          });
          this.currentQuestionType.set(state.question_type || 'guest_quiz');
          this.selectedAnswer.set(null);
          this.correctAnswer.set(null);
          this.answerSubmitted.set(false);
          this.iodGuess.set(null);
          this.phase.set('question');
          this.startCountdown(10);
        }
      },
      error: () => {}
    });
  }

  ngOnDestroy(): void {
    this.wakeLock.release();
    this.wsSub?.unsubscribe();
    this.pollSub?.unsubscribe();
    this.wsService.disconnect();
    this.stopCountdown();
    clearTimeout(this.boostTimer);
  }

  private handleWsMessage(msg: WsMessage): void {
    switch (msg.type) {
      case 'Connected': {
        console.log('Guest WS connected');
        break;
      }
      case 'QuestionStarted': {
        const q = msg as WsQuestionStarted;
        // Skip if we already have this round (from polling)
        if (q.round_id === this.lastSeenRoundId) break;
        this.lastSeenRoundId = q.round_id;
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
        this.currentQuestionType.set((q as any).question_type || 'guest_quiz');
        this.selectedAnswer.set(null);
        this.correctAnswer.set(null);
        this.answerSubmitted.set(false);
        this.iodGuess.set(null);
        this.phase.set('question');
        this.startCountdown(10);
        break;
      }
      case 'RoundClosed': {
        const rc = msg as WsRoundClosed;
        this.correctAnswer.set(rc.correct_answer);
        this.stopCountdown();
        // For ich_oder_du: show couple-answered phase (einig/uneinig)
        // For quiz: show round-result phase (richtig/falsch)
        if (this.currentQuestionType() === 'ich_oder_du') {
          this.phase.set('couple-answered');
        } else {
          this.phase.set('round-result');
        }
        // Fetch updated scores
        this.scoreService.getScores(this.code()).subscribe({
          next: (scores) => this.scores.set(scores),
          error: () => {}
        });
        break;
      }
      case 'IchOderDuStarted': {
        const iod = msg as WsIchOderDuStarted;
        this.ichOderDuText.set(iod.ich_oder_du_text);
        this.iodGuess.set(null);
        this.phase.set('ich-oder-du');
        break;
      }
      case 'CoupleAnswered': {
        const ca = msg as WsCoupleAnswered;
        this.coupleAnswer.set(ca.couple_answer);
        // Don't change phase here - wait for RoundClosed from admin clicking "Auflösen"
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
        this.scores.set(mapped);
        break;
      }
      case 'LuckyBoost': {
        const lb = msg as WsLuckyBoost;
        if (lb.player_id === this.playerId()) {
          this.luckyBoostMultiplier.set(lb.multiplier);
          this.luckyBoostVisible.set(true);
          clearTimeout(this.boostTimer);
          this.boostTimer = setTimeout(() => this.luckyBoostVisible.set(false), 5000);
        }
        break;
      }
      case 'GameEnded': {
        const ge = msg as WsGameEnded;
        console.log('Game ended', ge);
        this.phase.set('game-over');
        break;
      }
    }
  }

  submitAnswer(answer: string): void {
    if (this.answerSubmitted() || this.answerSubmitting()) return;

    this.selectedAnswer.set(answer);
    this.answerSubmitting.set(true);

    const req: AnswerRequest = {
      player_id: this.playerId(),
      player_name: this.playerName(),
      answer,
    };

    this.gameService.submitAnswer(this.code(), req).subscribe({
      next: () => {
        this.answerSubmitting.set(false);
        this.answerSubmitted.set(true);
        this.phase.set('answered');
      },
      error: (err) => {
        this.answerSubmitting.set(false);
        console.error('Answer submission error', err);
      }
    });
  }

  activateScreen(): void {
    this.screenActivated.set(true);
    this.wakeLock.acquire();
  }

  submitIodGuess(guess: 'ich' | 'du'): void {
    this.iodGuess.set(guess);
    // Submit guess to backend
    this.gameService.submitAnswer(this.code(), {
      player_id: this.playerId(),
      player_name: this.playerName(),
      answer: guess,
    }).subscribe({ error: () => {} });
  }

  private startCountdown(seconds: number = 10): void {
    this.stopCountdown();
    this.countdownWidth.set(100);
    const step = 100 / (seconds * 10);
    this.countdownInterval = setInterval(() => {
      const current = this.countdownWidth();
      if (current <= 0) {
        this.stopCountdown();
        if (this.phase() === 'question') {
          this.phase.set('answered');
          this.answerSubmitted.set(true);
        }
      } else {
        this.countdownWidth.set(Math.max(0, current - step));
      }
    }, 100);
  }

  private stopCountdown(): void {
    if (this.countdownInterval) {
      clearInterval(this.countdownInterval);
      this.countdownInterval = undefined;
    }
  }

  getAnswerBtnClass(option: string): string {
    const selected = this.selectedAnswer();
    const correct = this.correctAnswer();
    const submitted = this.answerSubmitted();

    if (correct !== null) {
      if (option === correct) return 'bg-green-600 border-green-500 correct-pop';
      if (option === selected && option !== correct) return 'bg-red-700 border-red-600 wrong-shake opacity-80';
      return 'bg-gray-800 border-gray-700 opacity-30';
    }

    if (submitted && option === selected) {
      return 'bg-rose-700 border-rose-600';
    }

    const colors: Record<string, string> = {
      A: 'bg-blue-700 border-blue-500',
      B: 'bg-emerald-700 border-emerald-500',
      C: 'bg-orange-600 border-orange-500',
      D: 'bg-purple-700 border-purple-500',
    };
    return colors[option] ?? 'bg-gray-800 border-gray-700';
  }

  getOptionText(option: string): string {
    const q = this.currentQuestion();
    if (!q) return '';
    const map: Record<string, string> = {
      A: q.option_a,
      B: q.option_b,
      C: q.option_c,
      D: q.option_d,
    };
    return map[option] ?? '';
  }
}
