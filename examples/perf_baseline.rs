use cellforge::Simulation;
use std::time::Instant;

struct Case<'a> {
    name: &'a str,
    mode: u8,
    birth: u32,
    survival: u32,
    states: u8,
    density: f32,
    seed: u32,
    steps_per_call: u32,
    calls: u32,
}

fn print_metrics(label: &str, total_steps: u64, elapsed_secs: f64, cells: f64) {
    let ticks_per_sec = total_steps as f64 / elapsed_secs;
    let cell_updates_per_sec = ticks_per_sec * cells;
    let ms_per_tick = (elapsed_secs * 1000.0) / total_steps as f64;
    println!(
        "{:<26} {:<18} {:>10.1} ticks/s  {:>9.3} ms/tick  {:>11.2} M cell-updates/s",
        label,
        "",
        ticks_per_sec,
        ms_per_tick,
        cell_updates_per_sec / 1_000_000.0
    );
}

fn run_case(case: &Case) {
    let mut sim = Simulation::new();
    sim.set_rule_mode(case.mode);
    sim.set_birth_rule(case.birth);
    sim.set_survival_rule(case.survival);
    sim.set_generations(case.states);
    sim.randomize_with_seed(case.density, case.seed);
    sim.refresh_pixels();

    // Warm up cache/code paths.
    for _ in 0..25 {
        sim.tick_no_render(case.steps_per_call);
    }

    let width = sim.get_width() as f64;
    let height = sim.get_height() as f64;
    let cells = width * height;

    let total_steps = case.steps_per_call as u64 * case.calls as u64;
    let started = Instant::now();
    for _ in 0..case.calls {
        sim.tick_no_render(case.steps_per_call);
    }
    let elapsed_tick_only = started.elapsed().as_secs_f64();

    let started = Instant::now();
    for _ in 0..case.calls {
        sim.tick_no_render(case.steps_per_call);
        sim.refresh_pixels();
    }
    let elapsed_with_refresh = started.elapsed().as_secs_f64();

    print_metrics(case.name, total_steps, elapsed_tick_only, cells);
    print_metrics("  + pixel refresh", total_steps, elapsed_with_refresh, cells);
}

fn main() {
    println!("CellForge core baseline benchmark");
    println!("grid: 1024x1024, release build");
    println!();

    let conway_life = Case {
        name: "Conway Life B3/S23",
        mode: 0,
        birth: 1 << 3,
        survival: (1 << 2) | (1 << 3),
        states: 2,
        density: 0.12,
        seed: 42,
        steps_per_call: 1,
        calls: 400,
    };

    let conway_maze = Case {
        name: "Conway Maze B3/S12345",
        mode: 0,
        birth: 1 << 3,
        survival: (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5),
        states: 2,
        density: 0.28,
        seed: 2025,
        steps_per_call: 1,
        calls: 400,
    };

    let generations_starwars = Case {
        name: "Generations StarWars",
        mode: 1,
        birth: 1 << 2,
        survival: (1 << 3) | (1 << 4) | (1 << 5),
        states: 4,
        density: 0.09,
        seed: 4040,
        steps_per_call: 1,
        calls: 400,
    };

    let generations_fireworld = Case {
        name: "Generations Fireworld",
        mode: 1,
        birth: 1 << 2,
        survival: (1 << 3) | (1 << 4),
        states: 8,
        density: 0.13,
        seed: 8080,
        steps_per_call: 1,
        calls: 400,
    };

    run_case(&conway_life);
    run_case(&conway_maze);
    run_case(&generations_starwars);
    run_case(&generations_fireworld);
}
