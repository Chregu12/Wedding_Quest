import WebSocket from 'ws';

/** Open a WebSocket and resolve once it is connected. */
export function connectWs(url: string, timeoutMs = 8000): Promise<WebSocket> {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    const timer = setTimeout(() => {
      ws.terminate();
      reject(new Error(`WS connect timeout: ${url}`));
    }, timeoutMs);
    ws.on('open', () => {
      clearTimeout(timer);
      resolve(ws);
    });
    ws.on('error', (err) => {
      clearTimeout(timer);
      reject(err);
    });
  });
}

/** Resolve with the first JSON message matching `predicate`. */
export function waitForMessage(
  ws: WebSocket,
  predicate: (msg: any) => boolean,
  timeoutMs = 8000,
): Promise<any> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      cleanup();
      reject(new Error('WS message timeout'));
    }, timeoutMs);
    const onMsg = (data: WebSocket.RawData) => {
      let parsed: any;
      try {
        parsed = JSON.parse(data.toString());
      } catch {
        return;
      }
      if (predicate(parsed)) {
        cleanup();
        resolve(parsed);
      }
    };
    const cleanup = () => {
      clearTimeout(timer);
      ws.off('message', onMsg);
    };
    ws.on('message', onMsg);
  });
}

/**
 * Attach a listener at connect time and return a live array that accumulates
 * every parsed message. Avoids the gap between sequential `waitForMessage`
 * calls — use with `expect.poll` to assert a message eventually arrives.
 */
export function collectWs(ws: WebSocket): any[] {
  const out: any[] = [];
  ws.on('message', (data: WebSocket.RawData) => {
    try {
      out.push(JSON.parse(data.toString()));
    } catch {
      /* ignore non-JSON frames */
    }
  });
  return out;
}

/**
 * Open a WebSocket while collecting messages from the very first frame.
 * The server sends CONNECTED immediately on connect, so attaching the listener
 * only after `open` can miss it — this captures it reliably.
 */
export function openAndCollect(
  url: string,
  timeoutMs = 8000,
): Promise<{ ws: WebSocket; messages: any[] }> {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    const messages: any[] = [];
    ws.on('message', (data: WebSocket.RawData) => {
      try {
        messages.push(JSON.parse(data.toString()));
      } catch {
        /* ignore non-JSON frames */
      }
    });
    const timer = setTimeout(() => {
      ws.terminate();
      reject(new Error(`WS connect timeout: ${url}`));
    }, timeoutMs);
    ws.on('open', () => {
      clearTimeout(timer);
      resolve({ ws, messages });
    });
    ws.on('error', (err) => {
      clearTimeout(timer);
      reject(err);
    });
  });
}

export function closeWs(ws: WebSocket | undefined) {
  try {
    ws?.close();
  } catch {
    /* ignore */
  }
}
