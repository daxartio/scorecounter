const CACHE_NAME = "scorecounter-cache-v2";

const BASE_PATH = (() => {
    const match = self.location.pathname.match(/^(.*)assets\/service-worker\.js$/);
    if (match && match[1]) {
        const path = match[1];
        return path.endsWith("/") ? path : `${path}/`;
    }
    return "/";
})();

const CORE_ASSETS = [
    BASE_PATH,
    `${BASE_PATH}index.html`,
    `${BASE_PATH}assets/manifest.webmanifest`,
    `${BASE_PATH}assets/icon-192.png`,
    `${BASE_PATH}assets/icon-512.png`,
    `${BASE_PATH}assets/apple-touch-icon.png`,
];

self.addEventListener("install", (event) => {
    event.waitUntil(
        caches
            .open(CACHE_NAME)
            .then((cache) => cache.addAll(CORE_ASSETS))
            .then(() => self.skipWaiting()),
    );
});

self.addEventListener("activate", (event) => {
    event.waitUntil(
        caches
            .keys()
            .then((keys) =>
                Promise.all(
                    keys
                        .filter((key) => key !== CACHE_NAME)
                        .map((key) => caches.delete(key)),
                ),
            ).then(() => self.clients.claim()),
    );
});

self.addEventListener("fetch", (event) => {
    if (event.request.method !== "GET") {
        return;
    }

    const url = new URL(event.request.url);
    if (url.origin !== self.location.origin || !url.pathname.startsWith(BASE_PATH)) {
        return;
    }

    event.respondWith(
        caches.match(event.request).then((cached) => {
            if (cached) {
                return cached;
            }
            return fetch(event.request)
                .then((response) => {
                    const cloned = response.clone();
                    caches.open(CACHE_NAME).then((cache) => {
                        cache.put(event.request, cloned);
                    });
                    return response;
                })
                .catch(() => cached);
        }),
    );
});
