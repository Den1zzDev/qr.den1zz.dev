const CACHE_NAME = 'qr-den1zz-v4';

self.addEventListener('install', () => {
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key))
      )
    ).then(() => self.clients.claim())
  );
});

function withTimeout(promise, ms) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('Network timeout')), ms);
    promise.then(
      (res) => {
        clearTimeout(timer);
        resolve(res);
      },
      (err) => {
        clearTimeout(timer);
        reject(err);
      }
    );
  });
}

function ensureWasmHeader(response, url) {
  if (url.endsWith('.wasm') && response.headers.get('content-type') !== 'application/wasm') {
    const headers = new Headers(response.headers);
    headers.set('content-type', 'application/wasm');
    return new Response(response.body, {
      status: response.status,
      statusText: response.statusText,
      headers,
    });
  }
  return response;
}

self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') return;
  const url = event.request.url;

  // Network-first strategy with 2500ms timeout for flaky connections
  const fetchOptions = url.startsWith(self.location.origin)
    ? { cache: 'no-cache' }
    : {};

  event.respondWith(
    withTimeout(fetch(event.request, fetchOptions), 2500)
      .then((networkResponse) => {
        if (
          networkResponse &&
          networkResponse.status === 200 &&
          url.startsWith(self.location.origin)
        ) {
          const responseToCache = networkResponse.clone();
          caches.open(CACHE_NAME).then((cache) => {
            cache.put(event.request, responseToCache);
          });
        }
        return ensureWasmHeader(networkResponse, url);
      })
      .catch(async () => {
        const cached = await caches.match(event.request);
        if (cached) {
          return ensureWasmHeader(cached, url);
        }
        if (event.request.mode === 'navigate') {
          const fallback = await caches.match('/');
          if (fallback) return fallback;
        }
        return new Response('Offline: Resource not available in cache', {
          status: 503,
          statusText: 'Service Unavailable',
          headers: { 'Content-Type': 'text/plain; charset=utf-8' },
        });
      })
  );
});
