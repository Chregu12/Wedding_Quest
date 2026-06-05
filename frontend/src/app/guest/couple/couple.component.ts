import { Component, inject, signal, OnInit, OnDestroy } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { CommonModule } from '@angular/common';
import { Subscription, interval } from 'rxjs';
import { switchMap } from 'rxjs/operators';
import { SessionService } from '../../services/session.service';
import { GameService } from '../../services/game.service';
import { WakeLockService } from '../../services/wakelock.service';
import { WebSocketService, WsMessage, WsQuestionStarted, WsIchOderDuStarted, WsRoundClosed } from '../../services/websocket.service';

type CouplePhase = 'waiting' | 'guest-answering' | 'my-turn' | 'answered' | 'result';

@Component({
  selector: 'app-couple-game',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './couple.component.html',
})
export class CoupleGameComponent implements OnInit, OnDestroy {
  private route = inject(ActivatedRoute);
  private sessionService = inject(SessionService);
  private gameService = inject(GameService);
  private wsService = inject(WebSocketService);
  private wakeLock = inject(WakeLockService);

  code = signal('');
  myName = signal('');
  partnerName = signal('');
  myPerson = signal<'a' | 'b'>('a');
  playerId = signal('');
  phase = signal<CouplePhase>('waiting');
  questionText = signal<string | null>(null);
  screenActivated = signal(false);
  questionType = signal<string>('guest_quiz');
  optionA = signal('');
  optionB = signal('');
  optionC = signal('');
  optionD = signal('');
  myAnswer = signal<string | null>(null);
  correctAnswer = signal<string | null>(null);

  private wsSub?: Subscription;
  private pollSub?: Subscription;
  private lastSeenRoundId = '';
  countdown = signal(10);
  private countdownInterval?: ReturnType<typeof setInterval>;

  ngOnInit(): void {
    this.wakeLock.acquire();
    this.code.set(this.route.snapshot.paramMap.get('code') ?? '');
    const name = this.route.snapshot.queryParamMap.get('name') ?? '';
    this.myName.set(name);

    this.sessionService.getByCode(this.code()).subscribe({
      next: (s) => {
        if (name.toLowerCase().trim() === s.person_a_name.toLowerCase().trim()) {
          this.partnerName.set(s.person_b_name);
          this.myPerson.set('a');
        } else {
          this.partnerName.set(s.person_a_name);
          this.myPerson.set('b');
        }
        console.log(`Couple: I am "${name}" → person ${this.myPerson()}`);
      }
    });

    // Auto-join as player (always try to get fresh ID)
    this.sessionService.join(this.code(), { display_name: name }).subscribe({
      next: (res) => {
        this.playerId.set(res.player_id);
        localStorage.setItem(`couple_player_id_${this.code()}_${name}`, res.player_id);
      },
      error: () => {
        // Already joined - fetch player list to get correct ID
        this.sessionService.getPlayers(this.code()).subscribe({
          next: (res) => {
            const me = res.players.find(p => p.display_name.toLowerCase() === name.toLowerCase());
            if (me) {
              this.playerId.set(me.id);
              localStorage.setItem(`couple_player_id_${this.code()}_${name}`, me.id);
            }
          }
        });
      }
    });

    if (this.code()) {
      this.wsService.connect(this.code());
      this.wsSub = this.wsService.messages().subscribe(msg => this.handleWsMessage(msg));

      // Snapshot the current round as "already seen" BEFORE polling, so a
      // question that already exists when this screen loads (a leftover round, or
      // one not (re)started for us) is never shown — only a round the admin
      // starts afterwards (a fresh round_id) appears.
      this.gameService.getState(this.code()).subscribe({
        next: (state) => {
          if (state.current_round_id) this.lastSeenRoundId = state.current_round_id;
          this.startStatePolling();
        },
        error: () => this.startStatePolling(),
      });
    }
  }

  private startStatePolling(): void {
    if (this.code()) {
      // Polling: check game state every 2s
      this.pollSub = interval(2000).pipe(
        switchMap(() => this.gameService.getState(this.code()))
      ).subscribe({
        next: (state) => {
          // Lobby reset between games: clear any stale question and return to the
          // waiting screen. 'waiting' is only ever set by an explicit game reset.
          if (state.status === 'waiting') {
            if (this.phase() !== 'waiting') {
              this.phase.set('waiting');
              this.questionText.set(null);
              this.lastSeenRoundId = '';
            }
            return;
          }
          if (state.status === 'question' && state.current_round_id && state.current_round_id !== this.lastSeenRoundId && state.question_text) {
            this.lastSeenRoundId = state.current_round_id;
            this.questionText.set(state.question_text);
            this.questionType.set(state.question_type || 'guest_quiz');
            this.optionA.set(state.option_a || '');
            this.optionB.set(state.option_b || '');
            this.optionC.set(state.option_c || '');
            this.optionD.set(state.option_d || '');
            this.myAnswer.set(null);
            this.correctAnswer.set(null);

            // Calculate elapsed time since round started
            const elapsed = state.started_at
              ? (Date.now() - new Date(state.started_at).getTime()) / 1000
              : 999;

            if (elapsed >= 10) {
              // Timer already expired - go directly to my-turn
              this.phase.set('my-turn');
            } else {
              // Show remaining countdown
              this.phase.set('guest-answering');
              this.countdown.set(Math.max(1, Math.ceil(10 - elapsed)));
              this.startGuestCountdown();
            }
          }
        },
        error: () => {}
      });
    }
  }

  ngOnDestroy(): void {
    this.wakeLock.release();
    this.wsSub?.unsubscribe();
    this.pollSub?.unsubscribe();
    this.wsService.disconnect();
    clearInterval(this.countdownInterval);
  }

  private handleWsMessage(msg: WsMessage): void {
    switch (msg.type) {
      case 'QuestionStarted': {
        const q = msg as WsQuestionStarted;
        this.questionText.set(q.question_text);
        this.questionType.set((q as any).question_type || 'guest_quiz');
        this.optionA.set(q.option_a);
        this.optionB.set(q.option_b);
        this.optionC.set(q.option_c);
        this.optionD.set(q.option_d);
        this.myAnswer.set(null);
        this.correctAnswer.set(null);
        this.phase.set('guest-answering');
        // Start 10s countdown, then it's our turn
        this.startGuestCountdown();
        break;
      }
      case 'IchOderDuStarted': {
        const iod = msg as WsIchOderDuStarted;
        this.questionText.set(iod.ich_oder_du_text);
        this.questionType.set('ich_oder_du');
        this.myAnswer.set(null);
        this.correctAnswer.set(null);
        this.phase.set('guest-answering');
        this.startGuestCountdown();
        break;
      }
      case 'RoundClosed': {
        const rc = msg as WsRoundClosed;
        this.correctAnswer.set(rc.correct_answer);
        this.phase.set('result');
        break;
      }
      case 'CoupleAnswered': {
        if (this.phase() === 'answered') {
          this.phase.set('result');
        }
        break;
      }
    }
  }

  private startGuestCountdown(): void {
    this.countdown.set(10);
    clearInterval(this.countdownInterval);
    this.countdownInterval = setInterval(() => {
      const c = this.countdown();
      if (c <= 1) {
        clearInterval(this.countdownInterval);
        this.countdown.set(0);
        this.phase.set('my-turn');
      } else {
        this.countdown.set(c - 1);
      }
    }, 1000);
  }

  answerQuiz(option: string): void {
    if (!this.playerId()) return; // Guard: wait for player registration
    this.myAnswer.set(option);
    this.phase.set('answered');
    // Submit as regular player answer with couple=true (full points)
    this.gameService.submitAnswer(this.code(), {
      player_id: this.playerId(),
      player_name: this.myName(),
      answer: option,
      couple: true,
    }).subscribe({ error: () => {} });
    // Also submit as couple individual answer (for couple-agree logic)
    this.gameService.submitCoupleIndividualAnswer(this.code(), this.myPerson(), option).subscribe({
      error: () => {}
    });
  }

  answerIchOderDu(choice: 'ich' | 'du'): void {
    if (!this.playerId()) return; // Guard: wait for player registration
    this.myAnswer.set(choice);
    this.phase.set('answered');

    // Convert relative choice (ich/du from MY perspective) to absolute (ich=personA, du=personB)
    // Person A: ich=ich, du=du (no change)
    // Person B: ich=du, du=ich (swap, because "ich" for B means B, but system "ich" means A)
    const systemAnswer = this.myPerson() === 'a' ? choice : (choice === 'ich' ? 'du' : 'ich');

    // Submit as regular player answer with couple=true (full points)
    this.gameService.submitAnswer(this.code(), {
      player_id: this.playerId(),
      player_name: this.myName(),
      answer: systemAnswer,
      couple: true,
    }).subscribe({ error: () => {} });
    // Also submit as couple individual answer
    this.gameService.submitCoupleIndividualAnswer(this.code(), this.myPerson(), systemAnswer).subscribe({
      error: () => {}
    });
  }

  activateScreen(): void {
    this.screenActivated.set(true);
    this.wakeLock.acquire();
  }
}
