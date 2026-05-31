import { Injectable } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class WakeLockService {
  private wakeLock: any = null;
  private keepAliveInterval: ReturnType<typeof setInterval> | null = null;
  private video: HTMLVideoElement | null = null;

  async acquire(): Promise<void> {
    // Method 1: Wake Lock API (Chrome, newer Safari 16.4+)
    try {
      if ('wakeLock' in navigator) {
        this.wakeLock = await (navigator as any).wakeLock.request('screen');
        document.addEventListener('visibilitychange', this.onVisibilityChange);
        return; // If Wake Lock API works, no need for fallback
      }
    } catch {
      // Not supported or denied - try fallback
    }

    // Method 2: Silent video loop (iOS Safari fallback)
    this.startVideoLoop();
  }

  release(): void {
    document.removeEventListener('visibilitychange', this.onVisibilityChange);
    if (this.wakeLock) {
      this.wakeLock.release();
      this.wakeLock = null;
    }
    this.stopVideoLoop();
  }

  private startVideoLoop(): void {
    if (this.video) return;

    try {
      // Create video element with a canvas-generated blob
      const canvas = document.createElement('canvas');
      canvas.width = 1;
      canvas.height = 1;
      const ctx = canvas.getContext('2d')!;
      ctx.fillRect(0, 0, 1, 1);

      // Use canvas.captureStream to create a media source (not supported on all iOS versions)
      if (!(canvas as any).captureStream) return;
      const stream = (canvas as any).captureStream(1); // 1 fps
      this.video = document.createElement('video');
      this.video.setAttribute('playsinline', '');
      this.video.setAttribute('muted', '');
      this.video.muted = true;
      this.video.style.position = 'fixed';
      this.video.style.top = '0';
      this.video.style.left = '0';
      this.video.style.width = '1px';
      this.video.style.height = '1px';
      this.video.style.opacity = '0.001';
      this.video.srcObject = stream;
      document.body.appendChild(this.video);

      this.video.play().catch(() => {});

      // Keep drawing to canvas to maintain stream
      const ctxRef = ctx;
      this.keepAliveInterval = setInterval(() => {
        ctxRef.fillStyle = ctxRef.fillStyle === '#000000' ? '#000001' : '#000000';
        ctxRef.fillRect(0, 0, 1, 1);
        if (this.video && this.video.paused) {
          this.video.play().catch(() => {});
        }
      }, 1000);
    } catch {
      // captureStream not supported on this browser - screen may lock
    }
  }

  private stopVideoLoop(): void {
    if (this.video) {
      this.video.pause();
      this.video.srcObject = null;
      this.video.remove();
      this.video = null;
    }
    if (this.keepAliveInterval) {
      clearInterval(this.keepAliveInterval);
      this.keepAliveInterval = null;
    }
  }

  private onVisibilityChange = async () => {
    if (document.visibilityState === 'visible') {
      if ('wakeLock' in navigator) {
        try {
          this.wakeLock = await (navigator as any).wakeLock.request('screen');
        } catch {
          // ignore
        }
      }
    }
  };
}
