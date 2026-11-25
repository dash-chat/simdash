// Client-only single page: everything is fetched after mount. Prerendering it,
// rather than emitting an SPA fallback, gives index.html relative asset paths,
// so the built frontend works from any directory of a static host.
export const ssr = false;
export const prerender = true;
