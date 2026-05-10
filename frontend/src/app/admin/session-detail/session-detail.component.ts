import { Component, inject, signal, computed, OnInit } from '@angular/core';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { CommonModule } from '@angular/common';
import { SessionService } from '../../services/session.service';
import { QuestionService } from '../../services/question.service';
import { Session } from '../../models/session.model';
import { Question, AddGuestQuizRequest, AddIchOderDuRequest } from '../../models/question.model';
import { ScoreConfig } from '../../models/score.model';
import { QRCodeModule } from 'angularx-qrcode';

type TabType = 'quiz' | 'ich-oder-du' | 'settings';

@Component({
  selector: 'app-session-detail',
  standalone: true,
  imports: [CommonModule, FormsModule, QRCodeModule, RouterLink],
  templateUrl: './session-detail.component.html',
})
export class SessionDetailComponent implements OnInit {
  private route = inject(ActivatedRoute);
  private router = inject(Router);
  private sessionService = inject(SessionService);
  private questionService = inject(QuestionService);

  code = signal('');
  session = signal<Session | null>(null);
  questions = signal<Question[]>([]);
  activeTab = signal<TabType>('quiz');
  loading = signal(false);
  error = signal<string | null>(null);
  successMessage = signal<string | null>(null);
  starting = signal(false);

  // QR Codes
  qrData = computed(() => `http://localhost:4200/join?code=${this.code()}`);
  displayUrl = computed(() => `http://localhost:4200/display/${this.code()}`);

  // Guest Quiz form
  quizForm = signal<AddGuestQuizRequest>({
    text: '',
    option_a: '',
    option_b: '',
    option_c: '',
    option_d: '',
    correct_answer: 'A',
    points: 100,
  });
  addingQuiz = signal(false);

  // Ich oder Du form
  ichOderDuForm = signal<AddIchOderDuRequest>({
    text: '',
    correct_answer: 'ich',
  });
  addingIchOderDu = signal(false);

  // Settings form
  scoreConfig = signal<ScoreConfig>({
    tier1_max_seconds: 10,
    tier2_max_seconds: 20,
    tier1_multiplier: 3,
    tier2_multiplier: 2,
    tier3_multiplier: 1,
    base_points: 100,
  });
  savingConfig = signal(false);

  quizQuestions = computed(() =>
    this.questions().filter(q => q.question_type === 'guest_quiz')
  );
  ichOderDuQuestions = computed(() =>
    this.questions().filter(q => q.question_type === 'ich_oder_du')
  );

  ngOnInit(): void {
    this.code.set(this.route.snapshot.paramMap.get('code') ?? '');
    this.loadSession();
    this.loadQuestions();
  }

  setTab(tab: TabType): void {
    this.activeTab.set(tab);
  }

  loadSession(): void {
    this.sessionService.getByCode(this.code()).subscribe({
      next: (s) => this.session.set(s),
      error: (err) => {
        this.error.set('Session nicht gefunden.');
        console.error(err);
      }
    });
  }

  loadQuestions(): void {
    this.questionService.getQuestions(this.code()).subscribe({
      next: (qs) => this.questions.set(qs),
      error: (err) => console.error('Fehler beim Laden der Fragen', err)
    });
  }

  updateQuizForm(field: keyof AddGuestQuizRequest, value: string | number): void {
    this.quizForm.set({ ...this.quizForm(), [field]: value });
  }

  updateIchOderDuForm(field: keyof AddIchOderDuRequest, value: string): void {
    this.ichOderDuForm.set({ ...this.ichOderDuForm(), [field]: value });
  }

  updateConfig(field: keyof ScoreConfig, value: number): void {
    this.scoreConfig.set({ ...this.scoreConfig(), [field]: value });
  }

  addQuizQuestion(): void {
    const form = this.quizForm();
    if (!form.text || !form.option_a || !form.option_b || !form.option_c || !form.option_d) {
      this.showError('Bitte alle Pflichtfelder ausfüllen.');
      return;
    }
    this.addingQuiz.set(true);
    this.questionService.addGuestQuiz(this.code(), form).subscribe({
      next: () => {
        this.addingQuiz.set(false);
        this.quizForm.set({ text: '', option_a: '', option_b: '', option_c: '', option_d: '', correct_answer: 'A', points: 100 });
        this.loadQuestions();
        this.showSuccess('Frage hinzugefügt!');
      },
      error: (err) => {
        this.addingQuiz.set(false);
        this.showError('Fehler beim Hinzufügen der Frage.');
        console.error(err);
      }
    });
  }

  addIchOderDu(): void {
    const form = this.ichOderDuForm();
    if (!form.text) {
      this.showError('Bitte Text eingeben.');
      return;
    }
    this.addingIchOderDu.set(true);
    this.questionService.addIchOderDu(this.code(), form).subscribe({
      next: () => {
        this.addingIchOderDu.set(false);
        this.ichOderDuForm.set({ text: '', correct_answer: 'ich' });
        this.loadQuestions();
        this.showSuccess('Ich-oder-Du Frage hinzugefügt!');
      },
      error: (err) => {
        this.addingIchOderDu.set(false);
        this.showError('Fehler beim Hinzufügen.');
        console.error(err);
      }
    });
  }

  deleteQuestion(id: string): void {
    if (!confirm('Frage wirklich löschen?')) return;
    this.questionService.deleteQuestion(this.code(), id).subscribe({
      next: () => {
        this.loadQuestions();
        this.showSuccess('Frage gelöscht.');
      },
      error: (err) => {
        this.showError('Fehler beim Löschen.');
        console.error(err);
      }
    });
  }

  saveConfig(): void {
    this.savingConfig.set(true);
    this.questionService.updateConfig(this.code(), this.scoreConfig()).subscribe({
      next: () => {
        this.savingConfig.set(false);
        this.showSuccess('Einstellungen gespeichert!');
      },
      error: (err) => {
        this.savingConfig.set(false);
        this.showError('Fehler beim Speichern der Einstellungen.');
        console.error(err);
      }
    });
  }

  startGame(): void {
    this.starting.set(true);
    this.sessionService.start(this.code()).subscribe({
      next: () => {
        this.starting.set(false);
        this.router.navigate(['/admin/sessions', this.code(), 'game']);
      },
      error: (err) => {
        this.starting.set(false);
        this.showError('Fehler beim Starten des Spiels.');
        console.error(err);
      }
    });
  }

  private showError(msg: string): void {
    this.error.set(msg);
    this.successMessage.set(null);
    setTimeout(() => this.error.set(null), 4000);
  }

  private showSuccess(msg: string): void {
    this.successMessage.set(msg);
    this.error.set(null);
    setTimeout(() => this.successMessage.set(null), 3000);
  }
}
