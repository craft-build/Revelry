/**
 * This is rendered as a desktop-style SPA. Disabling SSR keeps the runes-based
 * state model simple and matches the local-first philosophy from the spec —
 * there is no server, just a packaged app.
 */
export const ssr = false;
export const prerender = true;
