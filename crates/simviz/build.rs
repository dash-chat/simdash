//! Builds the SvelteKit frontend in `web/` and hands its output directory to
//! `src/server.rs`, which embeds it with `include_dir!`.
//!
//! Set `SIMVIZ_UI_DIR` to a directory holding an already-built frontend to skip
//! node entirely (CI without node, or a consumer that ships its own bundle).

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let web = manifest_dir.join("web");

    println!("cargo:rerun-if-env-changed=SIMVIZ_UI_DIR");
    for watched in [
        "src",
        "static",
        "package.json",
        "package-lock.json",
        "svelte.config.js",
        "vite.config.ts",
        "tsconfig.json",
    ] {
        println!("cargo:rerun-if-changed={}", web.join(watched).display());
    }

    if let Ok(prebuilt) = env::var("SIMVIZ_UI_DIR") {
        let dir = PathBuf::from(&prebuilt);
        assert!(
            dir.join("index.html").is_file(),
            "SIMVIZ_UI_DIR={prebuilt} has no index.html"
        );
        println!("cargo:rustc-env=SIMVIZ_UI_DIR={}", dir.display());
        return;
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("ui");
    ensure_node_modules(&web);
    run(
        Command::new("npm")
            .args(["run", "build"])
            .env("SIMVIZ_UI_OUT", &out)
            .current_dir(&web),
        "npm run build",
    );
    assert!(
        out.join("index.html").is_file(),
        "frontend build produced no index.html in {}",
        out.display()
    );
    println!("cargo:rustc-env=SIMVIZ_UI_DIR={}", out.display());
}

/// Run `npm ci` when node_modules is missing or older than the lockfile.
fn ensure_node_modules(web: &Path) {
    let lock = web.join("package-lock.json");
    let installed = web.join("node_modules").join(".package-lock.json");
    let fresh = match (installed.metadata(), lock.metadata()) {
        (Ok(i), Ok(l)) => i.modified().ok() >= l.modified().ok(),
        _ => false,
    };
    if !fresh {
        run(Command::new("npm").arg("ci").current_dir(web), "npm ci");
    }
}

fn run(cmd: &mut Command, what: &str) {
    let status = cmd.status().unwrap_or_else(|e| {
        panic!(
            "simviz: could not run `{what}` ({e}). The frontend needs node and npm to build; \
             or set SIMVIZ_UI_DIR to a prebuilt frontend directory."
        )
    });
    assert!(
        status.success(),
        "simviz: `{what}` failed with {status}. Set SIMVIZ_UI_DIR to skip the frontend build."
    );
}
