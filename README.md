# ternary-spiral

> Spiral-wave dynamics from Rock-Paper-Scissors cyclic dominance on a ternary lattice.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## What problem does this solve?

In ecology, three competing species often exhibit **cyclic dominance**: species A outcompetes B, B outcompetes C, and C outcompetes A. This non-transitive interaction is the classic Rock-Paper-Scissors (RPS) game. When such dynamics are placed on a spatial lattice with local dispersal, theory predicts the spontaneous formation of **spiral waves**—self-sustaining rotors that maintain biodiversity indefinitely.

Mathematically, the continuum limit is described by a set of reaction-diffusion PDEs (the Reichenbach-Lotka-Volterra system):

```
∂ρ_R/∂t = D∇²ρ_R + ρ_R(1 - ρ_R - σρ_P - (2-σ)ρ_S)
∂ρ_P/∂t = D∇²ρ_P + ρ_P(1 - ρ_P - σρ_S - (2-σ)ρ_R)
∂ρ_S/∂t = D∇²ρ_S + ρ_S(1 - ρ_S - σρ_R - (2-σ)ρ_P)
```

where `σ` controls the mobility. On a discrete grid, the same physics emerges from simple local update rules. This crate provides a fast, deterministic cellular-automaton implementation of those dynamics, together with ecological metrics (Shannon entropy, Simpson index, coexistence ratios) used in spatial biodiversity research.

---

## The science

### RPS dynamics

Each cell takes one of three states mapped to trits `{-1, 0, 1}`:

| Trit | State  | Beats     | Beaten by |
|------|--------|-----------|-----------|
| `-1` | Rock   | Scissors  | Paper     |
| ` 0` | Paper  | Rock      | Scissors  |
| ` 1` | Scissors | Paper   | Rock      |

The dominance graph is a directed 3-cycle. In a well-mixed (non-spatial) population, stochastic simulations show that one species eventually fixes; spatial structure is required for long-term coexistence.

### CA rules (invasion + majority)

At each discrete generation, every cell inspects its four von-Neumann neighbors (north, south, east, west) with toroidal wrapping. If a neighbor beats the focal cell, the focal cell **converts** to the majority species among those beaters. Ties break in the order Rock > Paper > Scissors. This rule captures the essence of local competitive exclusion followed by recolonization—the microscopic mechanism that nucleates spiral pairs at topological defects.

### Biodiversity indices

The crate computes standard ecological metrics:

- **Shannon entropy**: `H = -Σ p_i ln(p_i)` — measures community uncertainty.
- **Simpson index**: `λ = 1 - Σ p_i²` — probability that two randomly drawn cells are different species.
- **Evenness**: `J = H / ln(3)` — normalized Shannon entropy, ranging from 0 (monoculture) to 1 (perfect equipartition).

---

## Architecture

```text
┌─────────────────────────────────────────┐
│           SpatialGrid                   │
│  ┌─────┐ ┌─────┐ ┌─────┐               │
│  │Rock │ │Paper│ │Scissors│  ...       │
│  └─────┘ └─────┘ └─────┘               │
│  width × height  |  toroidal neighbors  │
└────────────┬────────────────────────────┘
             │ step()  (invasion + majority)
             ▼
┌─────────────────────────────────────────┐
│           SpiralWave                    │
│  generation, rock_count,                │
│  paper_count, scissors_count            │
│  ├── dominant_species()                 │
│  └── is_coexisting()  (>5% each)        │
└────────────┬────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────┐
│        BiodiversityIndex                │
│  shannon_entropy, simpson_index, evenness│
└─────────────────────────────────────────┘

Utilities:
  detect_invasion_fronts() ──► (x, y, attacker, defender)
  coexistence_metric()     ──► fraction of history with all 3 species
  run_simulation()         ──► Vec<SpiralWave>
```

---

## Getting Started

Add to `Cargo.toml`:

```toml
[dependencies]
ternary-spiral = { git = "https://github.com/SuperInstance/ternary-spiral.git" }
```

Run a 20-generation simulation and print biodiversity:

```rust
use ternary_spiral::{run_simulation, BiodiversityIndex};

fn main() {
    let history = run_simulation(40, 40, 20, 42);

    for wave in &history {
        println!(
            "gen {} | R:{} P:{} S:{}",
            wave.generation, wave.rock_count,
            wave.paper_count, wave.scissors_count
        );
    }

    // Biodiversity of the final generation, derived directly from its counts.
    let idx = BiodiversityIndex::from_wave(history.last().unwrap());
    println!("Shannon = {:.3}, Evenness = {:.3}", idx.shannon_entropy, idx.evenness);
}
```

Compile and run:

```bash
cargo run
```

---

## Running the Tests

```bash
cargo test
```

The unit test suite (plus a runnable doc-test) verifies every layer of the dynamics:

| Test | What it verifies |
|------|------------------|
| `rps_dominance` | The cyclic dominance relation: Rock→Scissors→Paper→Rock, with no self-beating. |
| `rps_from_trit` | Trit-to-cell conversion: `-1→Rock`, `0→Paper`, `1→Scissors`; invalid trits return `None`. |
| `rps_to_trit_roundtrip` | `to_trit()`/`from_trit()` round-trip for every state. |
| `rps_from_trit_out_of_range` | Boundary `i8` values (`-128, -2, 2, 3, 127`) are all rejected. |
| `grid_new_all_paper` | Fresh grids initialize to the neutral Paper state. |
| `grid_count` | `count()` correctly tallies species after manual `set()` operations. |
| `grid_neighbors_wrap` | Toroidal boundary conditions: neighbors wrap from `(0,0)` to the opposite edge. |
| `grid_step_invasion` | Invasion mechanics: a Scissors cell surrounded by Rock converts to Rock in one step. |
| `grid_empty_is_noop` | A `0×0` grid never panics on `step()` and yields all-zero biodiversity. |
| `grid_single_cell_is_stable` | On a `1×1` torus the cell is its own neighbors and never converts. |
| `spiral_wave_dominant` | `dominant_species()` correctly identifies a majority > 50 %. |
| `spiral_wave_coexistence` | `is_coexisting()` returns `true` only when all three species exceed the 5 % threshold. |
| `dominant_species_none_when_balanced` | No `>50 %` majority (and empty) → `dominant_species()` is `None`. |
| `biodiversity_uniform` | A monoculture yields Shannon entropy and Simpson index both ≈ 0. |
| `biodiversity_equal` | A perfectly mixed 1:1:1 grid maximizes Shannon entropy (~ln 3) and passes `is_diverse()`. |
| `biodiversity_hand_derived` | Non-uniform Shannon/Simpson/evenness match an independent hand derivation. |
| `biodiversity_from_wave_matches_compute` | `from_wave()` agrees with `compute()` for identical counts. |
| `invasion_fronts_detected` | `detect_invasion_fronts()` locates active competitive boundaries (e.g., Paper attacking Rock). |
| `coexistence_metric_all` | The coexistence metric returns 1.0 when every generation in a history contains all three species. |
| `coexistence_metric_empty_and_partial` | Empty history → `0.0`; a mixed history → `2/3`. |
| `run_simulation_length` | The high-level runner produces exactly the requested number of generational snapshots. |

---

## Related crates in the ternary ecosystem

- [`ternary-ising`](https://github.com/SuperInstance/ternary-ising) — 3-state Potts / Ising spin systems with spatial phase transitions.
- [`ternary-kuramoto`](https://github.com/SuperInstance/ternary-kuramoto) — Synchronization of coupled oscillators on a ternary state ring.
- [`ternary-wave`](https://github.com/SuperInstance/ternary-wave) — Discrete wave equations and excitable-media dynamics on ternary lattices.
- [`ternary-vortex`](https://github.com/SuperInstance/ternary-vortex) — Topological defect tracking and winding-number analysis in 2-D fields.

---

## License

This project is licensed under the [MIT License](LICENSE).
