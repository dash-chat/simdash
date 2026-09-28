import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
		// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
		runes: ({ filename }) => (filename.split(/[/\\]/).includes('node_modules') ? undefined : true)
	},
	kit: {
		// The app is a single client-rendered page. The `simviz` crate's build.rs builds it
		// into cargo's OUT_DIR (passed via SIMVIZ_UI_OUT) and embeds the result, so consumers
		// of simviz need no node toolchain at runtime.
		adapter: adapter({
			pages: process.env.SIMVIZ_UI_OUT ?? 'build',
			assets: process.env.SIMVIZ_UI_OUT ?? 'build',
			fallback: 'index.html',
			strict: true
		})
	}
};

export default config;
