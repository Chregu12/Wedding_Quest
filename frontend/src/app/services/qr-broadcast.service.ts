import { Injectable, signal } from '@angular/core';

export interface QrBroadcast {
  type: 'show' | 'hide';
  qrType?: 'guest' | 'couple';
  guestUrl?: string;
  coupleUrlA?: string;
  coupleUrlB?: string;
  personAName?: string;
  personBName?: string;
}

@Injectable({ providedIn: 'root' })
export class QrBroadcastService {
  private channel: BroadcastChannel;
  current = signal<QrBroadcast | null>(null);

  constructor() {
    this.channel = new BroadcastChannel('wedding-quest-qr');
    this.channel.onmessage = (event) => {
      this.current.set(event.data as QrBroadcast);
    };
  }

  show(data: QrBroadcast): void {
    this.channel.postMessage(data);
    this.current.set(data);
  }

  hide(): void {
    const msg: QrBroadcast = { type: 'hide' };
    this.channel.postMessage(msg);
    this.current.set(null);
  }
}
