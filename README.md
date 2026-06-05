# ternary-spiral

**Spiral waves from Rock-Paper-Scissors cyclic dominance on spatial grids.**

When three species compete cyclically (Rock beats Scissors beats Paper beats Rock) on a 2D grid, the result isn't random chaos — it's *self-organizing spiral waves*. Rock invades Scissors, Paper invades Rock, Scissors invades Paper, creating rotating spiral fronts where all three coexist indefinitely.

This is one of the most beautiful phenomena in mathematical biology, and it's inherently ternary: the cyclic group Z₃ is the engine that drives it.

---

## The Physics of Spiral Waves

RPS dynamics on a grid follow update rules like:
1. Pick a random cell and a random neighbor
2. If cell beats neighbor (cyclic), cell replaces neighbor
3. Repeat

The result: traveling wave fronts form spirals that rotate around defect points. These spirals:
- Are **stable** — they persist indefinitely without external input
- Maintain **biodiversity** — all three species survive at roughly equal frequency
- Have a characteristic **wavelength** determined by the competition rate

This is exactly what happens in:
- **Spatial ecology** — three competing species maintaining coexistence
- **Chemical reactions** — Belousov-Zhabotinsky oscillating reactions
- **Cardiac tissue** — spiral waves in heart muscle (fibrillation)
- **Excitable media** — forest fires, slime mold cycles

---

## Architecture

```
SpatialGrid (2D grid of RPSCells)
    │
    ├── RPSCell: Rock(-1), Paper(0), Scissors(+1)
    │   cyclic dominance: -1 beats +1, +1 beats 0, 0 beats -1
    │
    ├── SpiralWave simulation engine
    │   └── run_simulation(): step grid N times
    │
    ├── BiodiversityIndex
    │   └── Shannon diversity of the species distribution
    │
    ├── detect_invasion_fronts()
    │   └── Find boundary cells where different species meet
    │
    └── coexistence_metric()
        └── Measure long-term species balance
```

---

## Quick Start

```rust
use ternary_spiral::{SpatialGrid, SpiralWave, BiodiversityIndex, run_simulation};

let grid = SpatialGrid::random(50, 50, 42); // 50×50 grid, seed 42
let mut sim = SpiralWave::new(grid);

// Run 1000 generations
let history = run_simulation(&mut sim, 1000);

// Check biodiversity
let bio = BiodiversityIndex::from_grid(&sim.grid);
println!("Shannon diversity: {:.3}", bio.shannon());
println!("Species balance: {:?}", bio.counts());

// Detect spiral fronts
let fronts = detect_invasion_fronts(&sim.grid);
println!("Wave fronts: {} cells", fronts.len());
```

---

## Key Insight: Why Binary Can't Do This

Binary competitive systems (species A beats species B) always collapse to monoculture — one species dominates and the other goes extinct. Ternary cyclic competition (A beats B beats C beats A) has *no transitive ordering*, so no species can dominate permanently. The 0 state (middle species) is the key — it breaks the dominance hierarchy and enables coexistence.

This connects directly to our finding that **ternary systems have no phase transition**: the 0 state screens long-range order, preventing the system from locking into a single state. Spirals exist in a single "liquid" phase forever.

---

## Ecosystem

- **ternary-diehard** — Three-state cellular automata (related CA dynamics)
- **ternary-game** — RPS game theory
- **ternary-cell** — Cell-level ternary state machines
- **ternary-evo** — Evolutionary dynamics on ternary grids

## License

MIT
