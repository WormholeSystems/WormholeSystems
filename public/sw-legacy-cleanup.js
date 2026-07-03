/* Imported by the generated service worker. Deletes the cache left behind by
 * the legacy hand-written worker (removed August 2025) that served a stale
 * copy of the landing page cache-first; this worker replaces it at /sw.js.
 */
self.addEventListener('activate', (event) => {
    event.waitUntil(caches.delete('tunnelvision-v1'));
});
