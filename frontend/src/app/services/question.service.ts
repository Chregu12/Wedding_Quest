import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { Question, AddGuestQuizRequest, AddIchOderDuRequest } from '../models/question.model';
import { ScoreConfig } from '../models/score.model';

const SESSION_API = `http://${window.location.hostname}:3002`;

@Injectable({ providedIn: 'root' })
export class QuestionService {
  private http = inject(HttpClient);

  getQuestions(code: string): Observable<Question[]> {
    return this.http.get<Question[]>(`${SESSION_API}/sessions/${code}/questions`);
  }

  addGuestQuiz(code: string, request: AddGuestQuizRequest): Observable<Question> {
    return this.http.post<Question>(`${SESSION_API}/sessions/${code}/questions`, request);
  }

  addIchOderDu(code: string, request: AddIchOderDuRequest): Observable<Question> {
    return this.http.post<Question>(`${SESSION_API}/sessions/${code}/ich-oder-du`, request);
  }

  updateQuestion(code: string, questionId: string, data: Partial<Question>): Observable<Question> {
    return this.http.put<Question>(`${SESSION_API}/sessions/${code}/questions/${questionId}`, data);
  }

  deleteQuestion(code: string, questionId: string): Observable<void> {
    return this.http.delete<void>(`${SESSION_API}/sessions/${code}/questions/${questionId}`);
  }

  updateConfig(code: string, config: Partial<ScoreConfig>): Observable<void> {
    return this.http.put<void>(`${SESSION_API}/sessions/${code}/config`, config);
  }
}
