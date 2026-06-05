# From Binary to Ternary: Spiral Waves

## The Trap

Binary spatial dynamics model competition between two species: A beats B, or B beats A. The interaction graph is a directed edge — one winner, one loser. This topology always converges to fixation: one species eliminates the other. The only question is how fast.

But real ecosystems — and real complex systems — often have *three* players locked in a cycle. Rock beats Scissors, Scissors beats Paper, Paper beats Rock. This non-transitive cycle has no "winner." It self-organizes into beautiful spiral waves that maintain biodiversity indefinitely. Binary models can't capture this: they need an odd number of states for the cycle to close.

## Map to Three States

| Domain | −1 | 0 | +1 |
|--------|----|---|-----|
| RPS species | Rock (−1) | Paper (0) | Scissors (+1) |
| Beats | Scissors | Rock | Paper |
| Eaten by | Paper | Scissors | Rock |
| Lattice occupation | species A | species B | species C |

## From Binary to Ternary

**Before: two-species Lotka-Volterra**

```rust
enum Cell {
    SpeciesA,
    SpeciesB,
}
// A → B → A is impossible in 2 states
// (it would require A beats B beats A — a 2-cycle)
// With two species, one always eliminates the other
// Biodiversity collapses
```

**After: three-species RPS on a lattice**

```rust
#[derive(Clone, Copy)]
enum Cell {
    Rock(-1),     // beats Scissors
    Paper(0),     // beats Rock
    Scissors(1),  // beats Paper
}
// The 3-cycle closes: every species beats one and loses to one
// No species dominates
// On a spatial lattice, this produces spiral waves
```

**The emergence of spirals:**

Starting from a random configuration, the system spontaneously organizes. Patches of each species form. At the boundaries, the cyclic dominance creates rotational flows. Triple-points — where all three species meet — act as spiral cores, emitting waves of alternating species that propagate outward. The result is a self-sustaining pattern that maintains all three species indefinitely.

This *cannot happen* in binary. Two species on a lattice form sharp fronts — one pushes the other back until one vanishes. Three species create rotation, and rotation stabilizes coexistence.

**0 is not nothing:** In the spiral lattice, the Paper (0) species is not a "neutral" bystander. It's an active predator that beats Rock and is beaten by Scissors. The three states are locked in a perfect cycle where every state is equally important. There is no default, no background, no "nothing" cell — every cell is a full participant in the dynamics.

**From a binary modeler's perspective:**

```rust
// Binary: A beats B. That's it.
// You get fronts, not spirals.
// You get fixation, not coexistence.

// Ternary: A beats B, B beats C, C beats A.
// You get rotating fronts = spiral waves.
// You get indefinite coexistence.
```

The symmetry is the point. With an even number of states, cyclic dominance can't close. You need three states (or any odd number > 1) for the cycle to close on itself.

## Why It Matters

Ternary spatial dynamics produce emergent spiral waves that are impossible in binary. The cyclic dominance of rock-paper-scissors is the simplest mechanism that generates self-sustaining biodiversity on a lattice. This isn't a biological curiosity — it's a mathematical necessity. Two species always fixate. Three species can coexist. The step from binary to ternary is the step from death to life in spatial competition models.
