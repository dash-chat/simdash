use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

<<<<<<< HEAD
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
=======
>>>>>>> origin/router-handler-main
use clap::Parser;

use simdash_simulation::snapshot::{
    collect_all_op_ids, compute_delivery_rate, setup_from_input, SimulationSnapshot,
};

/// simdash CLI simulation runner
#[derive(Parser)]
struct Cli {
    /// Path to a JSON config file or a previous simulation snapshot to resume
    #[arg(default_value = "config.json")]
    input: PathBuf,

    /// Number of ticks to simulate (runs until Ctrl-C if omitted)
    #[arg(short, long)]
    ticks: Option<usize>,

    /// Output file path
    #[arg(short, long, default_value = "gen/snapshot.json")]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let setup = setup_from_input(&cli.input)?;
    let mut sim = setup.sim;
    let config = setup.config;
    let start_tick = setup.start_tick;
    let prior_elapsed = setup.prior_elapsed;

    let all_op_ids = collect_all_op_ids(sim.current_state());
    println!("Total unique ops in network: {}", all_op_ids.len());

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl-C handler");

    let target_ticks = cli.ticks.map(|t| start_tick + t);
    match target_ticks {
        Some(target) => println!(
            "Simulating ticks {}..{} ({} ticks)...",
            start_tick,
            target,
            target - start_tick
        ),
        None => println!("Simulating from tick {start_tick} until Ctrl-C..."),
    }

    let start = Instant::now();
    let mut tick = start_tick;
    let mut last_report = Instant::now();
    let report_interval = std::time::Duration::from_secs(1);

    while running.load(Ordering::SeqCst) {
        if let Some(target) = target_ticks {
            if tick >= target {
                break;
            }
        }

        sim.step_forward()?;
        tick += 1;

        if last_report.elapsed() >= report_interval {
            let elapsed = start.elapsed().as_secs_f64();
            let ticks_done = tick - start_tick;
            let rate = ticks_done as f64 / elapsed;
            let delivery = compute_delivery_rate(sim.current_state(), &all_op_ids);
            println!(
                "tick {tick} | delivery: {:.1}% | {:.1} ticks/s",
                delivery * 100.0,
                rate,
            );
            last_report = Instant::now();
        }
    }

    let session_elapsed = start.elapsed().as_secs_f64();
    let ticks_done = tick - start_tick;
    let session_rate = if ticks_done > 0 {
        ticks_done as f64 / session_elapsed
    } else {
        0.0
    };
    let final_delivery = compute_delivery_rate(sim.current_state(), &all_op_ids);
    let total_elapsed = prior_elapsed + session_elapsed;

    println!();
    println!("=== Simulation complete ===");
    println!("Ticks: {tick} (this session: {ticks_done})");
    println!("Elapsed: {total_elapsed:.2}s (this session: {session_elapsed:.2}s)");
    println!("Rate: {session_rate:.1} ticks/s");
    println!("Message delivery: {:.1}%", final_delivery * 100.0);

    let output =
        SimulationSnapshot::from_simulation(&sim, &all_op_ids, config, tick, total_elapsed);
    let json = serde_json::to_string_pretty(&output)?;
    std::fs::write(&cli.output, &json)?;
    println!("Output written to {}", cli.output.display());

    Ok(())
}
