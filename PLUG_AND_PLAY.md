# PLUG_AND_PLAY — ternary-spiral

> Spiral-wave dynamics from Rock-Paper-Scissors cyclic dominance on a ternary lattice.

This is a quick "pick up and run" guide. It mirrors the README but is trimmed to
the minimum you need to get a simulation producing numbers.

## What this crate does

`ternary-spiral` is a deterministic cellular automaton for three-species
**cyclic dominance** (Rock-Paper-Scissors) on a 2-D toroidal grid. Local
invasion plus a majority rule generate the self-organizing **spiral waves**
that keep all three species alive, and the crate reports standard ecological
metrics (Shannon entropy, Simpson index, evenness) over the run. Given a fixed
seed, every simulation is exactly reproducible.

This crate is *not* a function optimizer. There is no `SpiralOptimizer`; the
"spiral" refers to the spiral-wave patterns the RPS dynamics produce.

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ternary-spiral = { git = "https://github.com/SuperInstance/ternary-spiral" }
```

Run a simulation and read biodiversity off the final generation — no need to
rebuild the grid yourself:

```rust
use ternary_spiral::{run_simulation, BiodiversityIndex};

fn main() {
    // 20 generations on a 40x40 torus, seed 42 (deterministic).
    let history = run_simulation(40, 40, 20, 42);

    for wave in &history {
        println!(
            "gen {} | R:{} P:{} S:{}",
            wave.generation, wave.rock_count, wave.paper_count, wave.scissors_count
        );
    }

    // Biodiversity of the final generation, straight from its counts.
    let idx = BiodiversityIndex::from_wave(history.last().unwrap());
    println!("Shannon = {:.3}, Evenness = {:.3}", idx.shannon_entropy, idx.evenness);
}
```

Compile and run:

```bash
cargo run
```

## 🧩 Core building blocks

| Item | Role |
|------|------|
| [`RPSCell`] | The three cell states (`Rock`, `Paper`, `Scissors`) mapped to trits `{-1, 0, 1}`. |
| [`SpatialGrid`] | The `width × height` toroidal lattice; `step()` advances one generation. |
| [`SpiralWave`] | A per-generation population snapshot (`rock`/`paper`/`scissors` counts). |
| [`BiodiversityIndex`] | Shannon entropy, Simpson index and evenness; via `compute(grid)` or `from_wave`. |
| [`run_simulation`] | High-level helper: random grid → `steps` generations → `Vec<SpiralWave>`. |
| [`detect_invasion_fronts`] | Locates cells currently under attack by a dominant neighbor. |
| [`coexistence_metric`] | Fraction of a history in which all three species coexist. |

Want a hand-rolled grid instead of a random one? Build it directly:

```rust
use ternary_spiral::{SpatialGrid, RPSCell, SpiralWave, BiodiversityIndex};

let mut grid = SpatialGrid::new(10, 10); // starts all-Paper
grid.set(0, 0, RPSCell::Rock);
grid.set(5, 5, RPSCell::Scissors);
grid.step();
let idx = BiodiversityIndex::compute(&grid);
let wave = SpiralWave::from_grid(&grid, 1);
```

## 📚 Available Documentation

| Document | Description |
|----------|-------------|
| `README.md` | Full science background, CA rules, and architecture. |
| `docs/FROM_BINARY.md` | Why three states (not two) are needed for cyclic dominance. |

## 📄 License

MIT
