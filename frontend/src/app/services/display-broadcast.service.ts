import { Injectable, signal } from '@angular/core';

export interface DisplayOverlay {
  type: 'topic-choice' | 'question-preview' | 'game-over' | 'hide';
  topicA?: string;
  topicB?: string;
  questionText?: string;
  questionType?: string;
  optionA?: string;
  optionB?: string;
  optionC?: string;
  optionD?: string;
  personAName?: string;
  personBName?: string;
}

@Injectable({ providedIn: 'root' })
export class DisplayBroadcastService {
  private channel: BroadcastChannel;
  current = signal<DisplayOverlay | null>(null);

  constructor() {
    this.channel = new BroadcastChannel('wedding-quest-display');
    this.channel.onmessage = (event) => {
      const data = event.data as DisplayOverlay;
      if (data.type === 'hide') {
        this.current.set(null);
      } else {
        this.current.set(data);
      }
    };
  }

  showTopicChoice(topicA: string, topicB: string): void {
    const msg: DisplayOverlay = { type: 'topic-choice', topicA, topicB };
    this.channel.postMessage(msg);
    this.current.set(msg);
  }

  showQuestionPreview(data: { questionText: string; questionType: string; optionA?: string; optionB?: string; optionC?: string; optionD?: string; personAName?: string; personBName?: string }): void {
    const msg: DisplayOverlay = { type: 'question-preview', ...data };
    this.channel.postMessage(msg);
    this.current.set(msg);
  }

  showGameOver(): void {
    const msg: DisplayOverlay = { type: 'game-over' };
    this.channel.postMessage(msg);
    this.current.set(msg);
  }

  hide(): void {
    const msg: DisplayOverlay = { type: 'hide' };
    this.channel.postMessage(msg);
    this.current.set(null);
  }
}
