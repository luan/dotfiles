// Keep the pinned AAC handler; bound public authentication attempts at its edge.
import '/srv/sqm-server/slenderaac/build/shims.js';
import { handler } from '/srv/sqm-server/slenderaac/build/handler.js';
import { createServer } from 'node:http';

const windows = new Map();
const duration = 60_000;
const forms = new Set(['/account/login', '/account/signup', '/account/lost']);

function retryAfter(request) {
  if (request.method !== 'POST') return 0;
  const path = new URL(request.url, 'http://localhost').pathname;
  const limit = path === '/api/login' ? 120 : forms.has(path) ? 30 : 0;
  if (!limit) return 0;
  // Caddy overwrites this header; Node listens on loopback only.
  const address = request.headers['x-sqm-client-address'] || request.socket.remoteAddress;
  const key = `${address}:${limit}`;
  const now = Date.now();
  let window = windows.get(key);
  if (!window || window.expires <= now) {
    if (windows.size >= 4096) {
      for (const [key, value] of windows) {
        if (value.expires <= now) windows.delete(key);
      }
      if (windows.size >= 4096) return 60;
    }
    window = { expires: now + duration, count: 0 };
    windows.set(key, window);
  }
  window.count++;
  return window.count > limit ? Math.ceil((window.expires - now) / 1000) : 0;
}

const server = createServer(async (request, response) => {
  try {
    const retry = retryAfter(request);
    if (retry) {
      response.writeHead(429, {
        'Content-Type': 'text/plain; charset=utf-8',
        'Retry-After': String(retry),
        'Cache-Control': 'no-store',
      });
      response.end('Too many login attempts. Please try again in one minute.');
      return;
    }
    await handler(request, response);
  } catch {
    if (!response.headersSent) response.writeHead(500);
    response.end('Request failed.');
  }
});
server.requestTimeout = 30_000;
server.headersTimeout = 15_000;
server.listen(3080, '127.0.0.1', () => console.log('SlenderAAC listening on loopback:3080'));

for (const signal of ['SIGTERM', 'SIGINT']) {
  process.once(signal, () => {
    server.close(() => process.exit(0));
    setTimeout(() => process.exit(1), 30_000).unref();
  });
}
