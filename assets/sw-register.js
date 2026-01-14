if ("serviceWorker" in navigator) {
    window.addEventListener("load", () => {
        navigator.serviceWorker
            .register("/assets/service-worker.js", { scope: "/" })
            .catch((err) => console.error("SW registration failed", err));
    });
}
