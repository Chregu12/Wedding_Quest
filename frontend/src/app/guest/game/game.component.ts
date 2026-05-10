import { Component, inject, signal, computed, OnInit, OnDestroy } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { CommonModule } from '@angular/common';
import { Subscription } from 'rxjs';
import { GameService, AnswerRequest } from '../../services/game.service';
import { WebSocketService, WsMessage, WsQuestionStarted, WsRoundClosed, WsIchOderDuStarted, WsCoupleAnswered, WsScoresUpdated, WsGameEnded } from '../../services/websocket.service';
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
  private wsService = inject(WebSocketService);

  code = signal('');
  playerId = signal('');
  playerName = signal('');
  sessionId = signal('');

  phase = signal<GuestPhase>('waiting');
  currentQuestion = signal<CurrentQuestion | null>(null);
  selectedAnswer = signal<string | null>(null);
  correctAnswer = signal<string | null>(null);
  ichOderDuText = signal<string | null>(null);
  coupleAnswer = signal<string | null>(null);
  scores = signal<PlayerScore[]>([]);
  answerSubmitting = signal(false);
  answerSubmitted = signal(false);

  countdownWidth = signal(100);
  private countdownInterval?: ReturnType<typeof setInterval>;
  private wsSub?: Subscription;

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
    this.code.set(this.route.snapshot.paramMap.get('code') ?? '');
    this.playerId.set(localStorage.getItem('player_id') ?? '');
    this.playerName.set(localStorage.getItem('player_display_name') ?? '');
    this.sessionId.set(localStorage.getItem('session_id') ?? '');

    if (this.sessionId()) {
      this.wsService.connect(this.sessionId());
      this.wsSub = this.wsService.messages().subscribe(msg => this.handleWsMessage(msg));
    }
  }

  ngOnDestroy(): void {
    this.wsSub?.unsubscribe();
    this.wsService.disconnect();
    this.stopCountdown();
  }

  private handleWsMessage(msg: WsMessage): void {
    switch (msg.type) {
      case 'Connected': {
        console.log('Guest WS connected');
        break;
      }
      case 'QuestionStarted': {
        const q = msg as WsQuestionStarted;
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
        this.selectedAnswer.set(null);
        this.correctAnswer.set(null);
        this.answerSubmitted.set(false);
        this.phase.set('question');
        this.startCountdown(30);
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
        this.phase.set('couple-answered');
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
      case 'GameEnded': {
        const ge = msg as WsGameEnded;
        console.log('Game ended', ge);
        this.phase.set('game-over');
        break;
      }
    }
  }

  submitAnswer(answer: 'A' | 'B' | 'C' | 'D'): void {
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

  private startCountdown(seconds: number): void {
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
      // Round result phase
      if (option === correct) return 'bg-green-500 border-green-600 text-white scale-100';
      if (option === selected && option !== correct) return 'bg-red-400 border-red-500 text-white opacity-80';
      return 'bg-gray-100 border-gray-200 text-gray-400 opacity-50';
    }

    if (submitted && option === selected) {
      return 'bg-rose-500 border-rose-600 text-white';
    }

    const colors: Record<string, string> = {
      A: 'bg-blue-100 border-blue-300 text-blue-800 hover:bg-blue-200',
      B: 'bg-green-100 border-green-300 text-green-800 hover:bg-green-200',
      C: 'bg-orange-100 border-orange-300 text-orange-800 hover:bg-orange-200',
      D: 'bg-purple-100 border-purple-300 text-purple-800 hover:bg-purple-200',
    };
    return colors[option] ?? 'bg-gray-100 border-gray-300';
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
