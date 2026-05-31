import { Component, inject, signal, computed, OnInit, OnDestroy } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import { Subscription, interval } from 'rxjs';
import { switchMap } from 'rxjs/operators';
import { GameService, RoundInfo, CloseRoundResponse } from '../../services/game.service';
import { SessionService } from '../../services/session.service';
import { QuestionService } from '../../services/question.service';
import { ScoreService } from '../../services/score.service';
import { DisplayBroadcastService } from '../../services/display-broadcast.service';
import { WebSocketService, WsMessage, WsQuestionStarted, WsRoundClosed, WsScoresUpdated } from '../../services/websocket.service';
import { PlayerScore } from '../../models/score.model';

interface IodPair {
  pairIndex: number;
  topicA: string;
  topicB: string;
  typeA: string;
  typeB: string;
  questionA: { id: string; text: string; option_a?: string; option_b?: string; option_c?: string; option_d?: string; correct_answer: string };
  questionB: { id: string; text: string; option_a?: string; option_b?: string; option_c?: string; option_d?: string; correct_answer: string };
}

type GamePhase = 'starting' | 'question-ready' | 'question' | 'waiting-couple' | 'couple-done' | 'round-closed' | 'topic-choice' | 'couple-answered' | 'game-over';

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
  private sessionService = inject(SessionService);
  private scoreService = inject(ScoreService);
  private wsService = inject(WebSocketService);

  private questionService = inject(QuestionService);
  private displayBroadcast = inject(DisplayBroadcastService);

  code = signal('');
  sessionId = signal('');
  personAName = signal('');
  personBName = signal('');
  phase = signal<GamePhase>('starting');
  currentRound = signal<RoundInfo | null>(null);
  currentQuestionType = signal<string>('guest_quiz');
  closedRound = signal<CloseRoundResponse | null>(null);
  coupleAnswerA = signal<string | null>(null);
  coupleAnswerB = signal<string | null>(null);
  coupleAgree = signal<boolean>(false);
  scores = signal<PlayerScore[]>([]);
  error = signal<string | null>(null);
  loading = signal(false);
  countdown = signal(10);

  // Ich-oder-Du topic pairs
  iodPairs = signal<IodPair[]>([]);
  currentPairIndex = signal(0);
  currentTopicPair = signal<IodPair | null>(null);

  private wsSub?: Subscription;
  private scorePollSub?: Subscription;
  private initSubs: Subscription[] = [];
  private countdownTimer?: ReturnType<typeof setInterval>;

  ngOnInit(): void {
    this.code.set(this.route.snapshot.paramMap.get('code') ?? '');

    // Load couple names
    this.initSubs.push(this.sessionService.getByCode(this.code()).subscribe({
      next: (s) => {
        this.personAName.set(s.person_a_name);
        this.personBName.set(s.person_b_name);
      }
    }));

    // Load ALL topic pairs (quiz + ich-oder-du, grouped by type + pair_index)
    this.initSubs.push(this.questionService.getQuestions(this.code()).subscribe({
      next: (questions) => {
        const pairedQuestions = questions.filter(q => q.pair_index != null);
        const pairMap = new Map<string, any[]>();
        for (const q of pairedQuestions) {
          const key = `${q.question_type}_${q.pair_index}`;
          const arr = pairMap.get(key) || [];
          arr.push(q);
          pairMap.set(key, arr);
        }
        let pairs: IodPair[] = [];
        for (const [key, qs] of pairMap.entries()) {
          if (qs.length >= 2) {
            pairs.push({
              pairIndex: qs[0].pair_index!,
              topicA: qs[0].category || 'Thema A',
              topicB: qs[1].category || 'Thema B',
              typeA: qs[0].question_type,
              typeB: qs[1].question_type,
              questionA: { id: qs[0].id, text: qs[0].text, option_a: qs[0].option_a, option_b: qs[0].option_b, option_c: qs[0].option_c, option_d: qs[0].option_d, correct_answer: qs[0].correct_answer },
              questionB: { id: qs[1].id, text: qs[1].text, option_a: qs[1].option_a, option_b: qs[1].option_b, option_c: qs[1].option_c, option_d: qs[1].option_d, correct_answer: qs[1].correct_answer },
            });
          }
        }
        // Sort: alternating quiz and ich_oder_du (Quiz 1, IoD 1, Quiz 2, IoD 2, ...)
        const quizPairs = pairs.filter(p => p.typeA === 'guest_quiz').sort((a, b) => a.pairIndex - b.pairIndex);
        const iodPairs = pairs.filter(p => p.typeA === 'ich_oder_du').sort((a, b) => a.pairIndex - b.pairIndex);
        const interleaved: IodPair[] = [];
        const maxLen = Math.max(quizPairs.length, iodPairs.length);
        for (let i = 0; i < maxLen; i++) {
          if (i < quizPairs.length) interleaved.push(quizPairs[i]);
          if (i < iodPairs.length) interleaved.push(iodPairs[i]);
        }
        pairs = interleaved;
        this.iodPairs.set(pairs);
        // Show first topic choice after pairs are loaded
        this.showNextTopicChoice();
      }
    }));

    // Connect WebSocket using session code (matches Redis pub/sub channel)
    this.connectWebSocket(this.code());

    this.startPollingScores();
  }

  ngOnDestroy(): void {
    this.wsSub?.unsubscribe();
    this.scorePollSub?.unsubscribe();
    this.initSubs.forEach(s => s.unsubscribe());
    this.wsService.disconnect();
    this.stopCountdown();
  }

  private startCountdown(): void {
    this.stopCountdown();
    this.countdown.set(10);
    this.countdownTimer = setInterval(() => {
      const current = this.countdown();
      if (current <= 1) {
        this.stopCountdown();
        this.countdown.set(0);
        // After 10s: switch to waiting for couple answer
        if (this.phase() === 'question') {
          this.phase.set('waiting-couple');
        }
      } else {
        this.countdown.set(current - 1);
      }
    }, 1000);
  }

  private stopCountdown(): void {
    if (this.countdownTimer) {
      clearInterval(this.countdownTimer);
      this.countdownTimer = undefined;
    }
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
        // Only advance to question if we're not in a result/ich-oder-du phase
        if (this.phase() === 'starting' || this.phase() === 'couple-answered') {
          const q = msg as WsQuestionStarted;
          this.currentRound.set({
            round_id: q.round_id,
            question_text: q.question_text,
            option_a: q.option_a,
            option_b: q.option_b,
            option_c: q.option_c,
            option_d: q.option_d,
            correct_answer: q.correct_answer ?? '',
            round_number: q.round_number,
            total_questions: q.total_questions,
          });
          this.closedRound.set(null);
          this.phase.set('question');
          this.startCountdown();
        }
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
      case 'CoupleAnswered': {
        const ca = msg as import('../../services/websocket.service').WsCoupleAnswered;
        this.coupleAnswerA.set(ca.answer_a);
        this.coupleAnswerB.set(ca.answer_b);
        this.coupleAgree.set(ca.couple_answer !== 'uneinig');
        // Couple has answered - show "Auflösen" button for admin
        if (this.phase() === 'waiting-couple') {
          this.phase.set('couple-done');
        }
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

  private isFirstRound = true;

  startTimer(): void {
    if (this.loading()) return; // Prevent double-click
    const questionId = this.selectedQuestionId() || undefined;
    this.loading.set(true);
    this.displayBroadcast.hide();

    // Determine if first round or subsequent
    const call$ = this.isFirstRound
      ? this.gameService.startGame(this.code(), questionId)
      : this.gameService.nextQuestion(this.code(), questionId);

    if (this.isFirstRound) {
      this.scoreService.resetScores(this.code()).subscribe();
      this.isFirstRound = false;
    }

    call$.subscribe({
      next: (round: any) => {
        this.loading.set(false);
        if (round) {
          this.currentRound.set(round);
          this.phase.set('question');
          this.startCountdown();
        } else {
          this.phase.set('game-over');
        }
      },
      error: () => {
        this.loading.set(false);
        this.error.set('Fehler beim Starten der Runde.');
      }
    });
  }

  showNextTopicChoice(): void {
    const pairs = this.iodPairs();
    const idx = this.currentPairIndex();
    if (idx < pairs.length) {
      this.currentTopicPair.set(pairs[idx]);
      this.phase.set('topic-choice');
      // Show topics on beamer
      this.displayBroadcast.showTopicChoice(pairs[idx].topicA, pairs[idx].topicB);
    } else {
      this.phase.set('game-over');
    }
  }

  startFirstRound(): void {
    this.loading.set(true);
    // Reset scores for a fresh game
    this.scoreService.resetScores(this.code()).subscribe();
    this.gameService.startGame(this.code()).subscribe({
      next: (round) => {
        this.loading.set(false);
        this.currentRound.set(round);
        this.phase.set('question');
        this.startCountdown();
      },
      error: (err) => {
        this.loading.set(false);
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
    this.stopCountdown();
    this.loading.set(true);
    this.gameService.closeRound(this.code()).subscribe({
      next: (result) => {
        this.loading.set(false);
        this.closedRound.set(result);
        // Always show the solution first
        this.phase.set('round-closed');
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


  selectedQuestionId = signal<string | null>(null);

  selectTopic(choice: 'A' | 'B'): void {
    const pair = this.currentTopicPair();
    if (!pair) return;

    const chosen = choice === 'A' ? pair.questionA : pair.questionB;

    // Advance pair index for next time
    this.currentPairIndex.set(this.currentPairIndex() + 1);

    // Hide topic choice on beamer
    this.displayBroadcast.hide();

    // Store chosen question id and type - backend call happens on "Timer starten"
    this.selectedQuestionId.set(chosen.id);
    const chosenType = choice === 'A' ? pair.typeA : pair.typeB;
    this.currentQuestionType.set(chosenType);

    // Show question preview in admin
    this.currentRound.set({
      round_id: '',
      question_text: chosen.text,
      option_a: chosen.option_a || '',
      option_b: chosen.option_b || '',
      option_c: chosen.option_c || '',
      option_d: chosen.option_d || '',
      correct_answer: chosen.correct_answer || '',
      round_number: this.currentPairIndex(),
      total_questions: this.iodPairs().length,
    });
    this.closedRound.set(null);
    this.coupleAnswerA.set(null);
    this.coupleAnswerB.set(null);
    this.coupleAgree.set(false);
    this.phase.set('question-ready');

    // Show question on beamer (without timer)
    this.displayBroadcast.showQuestionPreview({
      questionText: chosen.text,
      questionType: chosenType,
      optionA: chosen.option_a || '',
      optionB: chosen.option_b || '',
      optionC: chosen.option_c || '',
      optionD: chosen.option_d || '',
      personAName: this.personAName(),
      personBName: this.personBName(),
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
          this.startCountdown();
        } else {
          this.phase.set('game-over');
        }
      },
      error: (err) => {
        this.loading.set(false);
        if (err.status === 204) {
          this.phase.set('game-over');
        } else {
          this.error.set('Fehler beim Laden der nächsten Frage.');
          console.error(err);
        }
      }
    });
  }

  getOptionLabel(option: string): string {
    const map: Record<string, string> = { A: 'A', B: 'B', C: 'C', D: 'D' };
    return map[option] ?? option;
  }

  isCorrectOption(option: string): boolean {
    if (this.phase() !== 'round-closed') return false;
    const round = this.currentRound();
    return round !== null && round.correct_answer === option;
  }

  getOptionClass(option: string): string {
    const closed = this.closedRound();
    const round = this.currentRound();
    const isCorrect = this.phase() === 'round-closed' && round && round.correct_answer === option;

    if (closed) {
      if (closed.correct_answer === option) {
        return 'bg-green-200 border-green-500 text-green-900 ring-2 ring-green-400';
      }
      return 'bg-gray-100 border-gray-200 text-gray-500 opacity-60';
    }

    // During active question (not question-ready): highlight correct answer for moderator
    if (isCorrect) {
      return 'bg-green-100 border-green-400 text-green-800 ring-1 ring-green-300';
    }
    const colors: Record<string, string> = {
      A: 'bg-blue-100 border-blue-300 text-blue-800',
      B: 'bg-green-100 border-green-300 text-green-800',
      C: 'bg-orange-100 border-orange-300 text-orange-800',
      D: 'bg-purple-100 border-purple-300 text-purple-800',
    };
    return colors[option] ?? 'bg-gray-100 border-gray-300';
  }

  endGame(): void {
    if (!confirm('Spiel wirklich beenden?')) return;
    this.stopCountdown();
    this.displayBroadcast.showGameOver();
    this.router.navigate(['/admin/sessions', this.code()]);
  }

  goToSetup(): void {
    this.router.navigate(['/admin/sessions', this.code()]);
  }
}
