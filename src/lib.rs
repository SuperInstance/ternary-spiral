//! Spiral-wave dynamics from Rock-Paper-Scissors (RPS) cyclic dominance on a
//! ternary lattice.
//!
//! Three species compete non-transitively — Rock beats Scissors, Scissors beats
//! Paper, Paper beats Rock. Placed on a spatial grid with local dispersal, this
//! generates the self-organizing **spiral waves** that sustain long-term
//! biodiversity: the canonical finding of spatial RPS ecology.
//!
//! # When to use this
//!
//! Use this crate to study or demonstrate spatial cyclic dominance — how local
//! invasion plus a majority rule nucleate rotating spiral patterns, and how
//! biodiversity indices (Shannon entropy, Simpson index, evenness) evolve over
//! generations. The automaton is fully deterministic given a seed, so every
//! simulation is exactly reproducible.
//!
//! # Quick example
//!
//! ```
//! use ternary_spiral::{run_simulation, BiodiversityIndex};
//!
//! // 30 generations on a 30x30 torus, seed 42 (deterministic).
//! let history = run_simulation(30, 30, 30, 42);
//! let final_wave = history.last().unwrap();
//! let idx = BiodiversityIndex::from_wave(final_wave);
//!
//! // The cell count is conserved by the update rule.
//! assert_eq!(final_wave.total_cells(), 30 * 30);
//! // A well-mixed state sits near the maximum entropy ln(3).
//! assert!(idx.shannon_entropy > 0.0 && idx.shannon_entropy.is_finite());
//! ```

#![deny(missing_docs)]

/// The three competing cell states of the Rock-Paper-Scissors cycle.
///
/// States are mapped onto balanced trits `{-1, 0, 1}` so the dominance relation
/// forms a directed 3-cycle: each species beats exactly one other and is beaten
/// by the remaining one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RPSCell {
    /// Rock (trit `-1`); beats [`Scissors`](RPSCell::Scissors).
    Rock = -1,
    /// Paper (trit `0`); beats [`Rock`](RPSCell::Rock).
    Paper = 0,
    /// Scissors (trit `1`); beats [`Paper`](RPSCell::Paper).
    Scissors = 1,
}

impl RPSCell {
    /// Rock beats Scissors, Scissors beats Paper, Paper beats Rock.
    pub fn beats(&self, other: &RPSCell) -> bool {
        matches!(
            (self, other),
            (RPSCell::Rock, RPSCell::Scissors)
                | (RPSCell::Scissors, RPSCell::Paper)
                | (RPSCell::Paper, RPSCell::Rock)
        )
    }

    /// Convert trit value to RPSCell: -1 -> Rock, 0 -> Paper, 1 -> Scissors.
    pub fn from_trit(t: i8) -> Option<RPSCell> {
        match t {
            -1 => Some(RPSCell::Rock),
            0 => Some(RPSCell::Paper),
            1 => Some(RPSCell::Scissors),
            _ => None,
        }
    }

    /// Returns the trit value (`-1`, `0` or `1`) of this cell, the inverse of
    /// [`from_trit`](RPSCell::from_trit).
    pub fn to_trit(&self) -> i8 {
        *self as i8
    }
}

/// A two-dimensional lattice of [`RPSCell`]s with toroidal (wrap-around)
/// boundaries.
///
/// Coordinates are `(x, y)` with `x` in `0..width` and `y` in `0..height`.
/// Neighbors wrap across edges so every cell has exactly four von-Neumann
/// neighbors.
pub struct SpatialGrid {
    /// Number of columns.
    pub width: usize,
    /// Number of rows.
    pub height: usize,
    /// Row-major cell storage: `cells[y][x]`.
    pub cells: Vec<Vec<RPSCell>>,
}

impl SpatialGrid {
    /// Create a new grid initialized to all Paper.
    pub fn new(width: usize, height: usize) -> Self {
        SpatialGrid {
            width,
            height,
            cells: vec![vec![RPSCell::Paper; width]; height],
        }
    }

    /// Create a random grid using a simple LCG PRNG seeded with `seed`.
    pub fn random(width: usize, height: usize, seed: u64) -> Self {
        let mut state = seed;
        let mut cells = vec![vec![RPSCell::Paper; width]; height];
        for row in cells.iter_mut() {
            for cell in row.iter_mut() {
                // LCG: multiplier and increment from Numerical Recipes
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                let trit = ((state >> 30) % 3) as i8 - 1; // gives -1, 0, or 1
                *cell = RPSCell::from_trit(trit).unwrap_or(RPSCell::Paper);
            }
        }
        SpatialGrid {
            width,
            height,
            cells,
        }
    }

    /// Returns the cell at column `x`, row `y`.
    ///
    /// # Panics
    ///
    /// Panics if `(x, y)` is outside `0..width` × `0..height`.
    pub fn get(&self, x: usize, y: usize) -> RPSCell {
        self.cells[y][x]
    }

    /// Overwrites the cell at column `x`, row `y`.
    ///
    /// # Panics
    ///
    /// Panics if `(x, y)` is outside `0..width` × `0..height`.
    pub fn set(&mut self, x: usize, y: usize, cell: RPSCell) {
        self.cells[y][x] = cell;
    }

    /// Return 4-connected neighbors with wrapping.
    pub fn neighbors(&self, x: usize, y: usize) -> Vec<RPSCell> {
        let left = if x == 0 { self.width - 1 } else { x - 1 };
        let right = if x + 1 >= self.width { 0 } else { x + 1 };
        let up = if y == 0 { self.height - 1 } else { y - 1 };
        let down = if y + 1 >= self.height { 0 } else { y + 1 };

        vec![
            self.cells[y][left],
            self.cells[y][right],
            self.cells[up][x],
            self.cells[down][x],
        ]
    }

    /// Advance one generation: each cell that has a neighbor which beats it
    /// becomes the type of the most frequent beater among its neighbors
    /// (majority rule among beaters; ties broken by order Rock > Paper > Scissors).
    pub fn step(&mut self) {
        let old = self.cells.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let current = old[y][x];
                let left = if x == 0 { self.width - 1 } else { x - 1 };
                let right = if x + 1 >= self.width { 0 } else { x + 1 };
                let up = if y == 0 { self.height - 1 } else { y - 1 };
                let down = if y + 1 >= self.height { 0 } else { y + 1 };

                let neighbor_cells = [old[y][left], old[y][right], old[up][x], old[down][x]];

                let beaters: Vec<RPSCell> = neighbor_cells
                    .iter()
                    .copied()
                    .filter(|n| n.beats(&current))
                    .collect();

                if beaters.is_empty() {
                    continue;
                }

                // Majority rule among beaters
                let mut rock_count = 0usize;
                let mut paper_count = 0usize;
                let mut scissors_count = 0usize;
                for b in &beaters {
                    match b {
                        RPSCell::Rock => rock_count += 1,
                        RPSCell::Paper => paper_count += 1,
                        RPSCell::Scissors => scissors_count += 1,
                    }
                }

                let winner = if rock_count >= paper_count && rock_count >= scissors_count {
                    RPSCell::Rock
                } else if paper_count >= scissors_count {
                    RPSCell::Paper
                } else {
                    RPSCell::Scissors
                };

                if winner.beats(&current) {
                    self.cells[y][x] = winner;
                }
            }
        }
    }

    /// Count cells of a given type.
    pub fn count(&self, cell: RPSCell) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&c| c == cell)
            .count()
    }
}

/// A population snapshot: how many of each species are present at a given
/// [`generation`](SpiralWave::generation).
///
/// Built from a [`SpatialGrid`] via [`SpiralWave::from_grid`], or produced per
/// step by [`run_simulation`]. It carries the counts needed for coexistence and
/// dominance analysis without retaining the full grid.
pub struct SpiralWave {
    /// Generation index (1-based when produced by [`run_simulation`]).
    pub generation: u64,
    /// Number of Rock cells.
    pub rock_count: usize,
    /// Number of Paper cells.
    pub paper_count: usize,
    /// Number of Scissors cells.
    pub scissors_count: usize,
}

impl SpiralWave {
    /// Build a snapshot from a grid, recording the population counts at the
    /// given generation.
    pub fn from_grid(grid: &SpatialGrid, generation: u64) -> Self {
        SpiralWave {
            generation,
            rock_count: grid.count(RPSCell::Rock),
            paper_count: grid.count(RPSCell::Paper),
            scissors_count: grid.count(RPSCell::Scissors),
        }
    }

    /// Returns the dominant species if one has >50% of cells, otherwise None.
    pub fn dominant_species(&self) -> Option<RPSCell> {
        let total = self.total_cells();
        if total == 0 {
            return None;
        }
        let half = total / 2;
        if self.rock_count > half {
            Some(RPSCell::Rock)
        } else if self.paper_count > half {
            Some(RPSCell::Paper)
        } else if self.scissors_count > half {
            Some(RPSCell::Scissors)
        } else {
            None
        }
    }

    /// Total number of cells across all three species.
    pub fn total_cells(&self) -> usize {
        self.rock_count + self.paper_count + self.scissors_count
    }

    /// Returns true if all three species are present and each exceeds 5% of total.
    pub fn is_coexisting(&self) -> bool {
        let total = self.total_cells();
        if total == 0 {
            return false;
        }
        let threshold = total / 20; // 5%
        self.rock_count > threshold
            && self.paper_count > threshold
            && self.scissors_count > threshold
    }
}

/// Standard ecological diversity indices for a three-species community.
///
/// Computed from species proportions via [`BiodiversityIndex::compute`] (from a
/// grid) or [`BiodiversityIndex::from_wave`] (from a snapshot). All fields are
/// `0.0` for an empty population.
pub struct BiodiversityIndex {
    /// Shannon entropy `H = -Σ p_i ln(p_i)`; `0` for a monoculture, `ln 3` for
    /// a perfectly even three-species mix.
    pub shannon_entropy: f64,
    /// Simpson index `λ = 1 - Σ p_i²`; the probability two randomly drawn cells
    /// differ in species. Ranges `0` (monoculture) to `2/3` (even mix).
    pub simpson_index: f64,
    /// Pielou's evenness `J = H / ln(3)`; Shannon entropy normalized to
    /// `[0, 1]`.
    pub evenness: f64,
}

impl BiodiversityIndex {
    /// Compute the indices from a spatial grid.
    ///
    /// For an empty grid all three fields are `0.0`. The cell count is formed
    /// as `(width as f64) * (height as f64)` rather than `(width * height)` so
    /// that very large grids cannot overflow the `usize` intermediate before
    /// the cast to `f64`.
    pub fn compute(grid: &SpatialGrid) -> Self {
        let total = (grid.width as f64) * (grid.height as f64);
        let counts = [
            grid.count(RPSCell::Rock) as f64,
            grid.count(RPSCell::Paper) as f64,
            grid.count(RPSCell::Scissors) as f64,
        ];
        Self::from_counts(&counts, total)
    }

    /// Compute the indices directly from a [`SpiralWave`] snapshot, without
    /// needing access to the underlying grid.
    ///
    /// This is useful after [`run_simulation`], which returns counts but
    /// discards the final grid — callers can obtain biodiversity without having
    /// to reconstruct and re-step the whole lattice.
    pub fn from_wave(wave: &SpiralWave) -> Self {
        let total = wave.total_cells() as f64;
        let counts = [
            wave.rock_count as f64,
            wave.paper_count as f64,
            wave.scissors_count as f64,
        ];
        Self::from_counts(&counts, total)
    }

    /// Shared core: Shannon entropy, Simpson index and evenness from species
    /// counts and a total population. An empty population (total `<= 0`) yields
    /// all-zero indices.
    fn from_counts(counts: &[f64; 3], total: f64) -> Self {
        if total <= 0.0 {
            return BiodiversityIndex {
                shannon_entropy: 0.0,
                simpson_index: 0.0,
                evenness: 0.0,
            };
        }

        let mut shannon = 0.0f64;
        let mut sum_sq = 0.0f64;
        for &c in counts {
            let p = c / total;
            if p > 0.0 {
                shannon -= p * p.ln();
            }
            sum_sq += p * p;
        }

        let ln3 = 3.0f64.ln();
        let evenness = if ln3 > 0.0 { shannon / ln3 } else { 0.0 };

        BiodiversityIndex {
            shannon_entropy: shannon,
            simpson_index: 1.0 - sum_sq,
            evenness,
        }
    }

    /// Returns true if shannon_entropy > 0.5 * ln(3).
    pub fn is_diverse(&self) -> bool {
        self.shannon_entropy > 0.5 * 3.0f64.ln()
    }
}

/// Returns a list of (x, y, attacker, defender) for cells where an attacking
/// neighbor beats the current cell.
pub fn detect_invasion_fronts(grid: &SpatialGrid) -> Vec<(usize, usize, RPSCell, RPSCell)> {
    let mut fronts = Vec::new();
    for y in 0..grid.height {
        for x in 0..grid.width {
            let current = grid.get(x, y);
            for neighbor in grid.neighbors(x, y) {
                if neighbor.beats(&current) {
                    fronts.push((x, y, neighbor, current));
                    break; // one entry per cell
                }
            }
        }
    }
    fronts
}

/// Returns the fraction of generations where all three species coexisted.
pub fn coexistence_metric(history: &[SpiralWave]) -> f64 {
    if history.is_empty() {
        return 0.0;
    }
    let coexisting = history.iter().filter(|w| w.is_coexisting()).count();
    coexisting as f64 / history.len() as f64
}

/// Run the full simulation and return a snapshot after each step.
pub fn run_simulation(width: usize, height: usize, steps: usize, seed: u64) -> Vec<SpiralWave> {
    let mut grid = SpatialGrid::random(width, height, seed);
    let mut snapshots = Vec::with_capacity(steps);
    for gen in 0..steps {
        grid.step();
        snapshots.push(SpiralWave::from_grid(&grid, gen as u64 + 1));
    }
    snapshots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rps_dominance() {
        assert!(RPSCell::Rock.beats(&RPSCell::Scissors));
        assert!(RPSCell::Scissors.beats(&RPSCell::Paper));
        assert!(RPSCell::Paper.beats(&RPSCell::Rock));
        // Negative cases
        assert!(!RPSCell::Rock.beats(&RPSCell::Paper));
        assert!(!RPSCell::Scissors.beats(&RPSCell::Rock));
        assert!(!RPSCell::Paper.beats(&RPSCell::Scissors));
        // Self case
        assert!(!RPSCell::Rock.beats(&RPSCell::Rock));
    }

    #[test]
    fn rps_from_trit() {
        assert_eq!(RPSCell::from_trit(-1), Some(RPSCell::Rock));
        assert_eq!(RPSCell::from_trit(0), Some(RPSCell::Paper));
        assert_eq!(RPSCell::from_trit(1), Some(RPSCell::Scissors));
        assert_eq!(RPSCell::from_trit(2), None);
    }

    #[test]
    fn grid_new_all_paper() {
        let grid = SpatialGrid::new(5, 5);
        for y in 0..5 {
            for x in 0..5 {
                assert_eq!(grid.get(x, y), RPSCell::Paper);
            }
        }
        assert_eq!(grid.count(RPSCell::Paper), 25);
        assert_eq!(grid.count(RPSCell::Rock), 0);
        assert_eq!(grid.count(RPSCell::Scissors), 0);
    }

    #[test]
    fn grid_count() {
        let mut grid = SpatialGrid::new(4, 4);
        grid.set(0, 0, RPSCell::Rock);
        grid.set(1, 1, RPSCell::Rock);
        grid.set(2, 2, RPSCell::Scissors);
        assert_eq!(grid.count(RPSCell::Rock), 2);
        assert_eq!(grid.count(RPSCell::Scissors), 1);
        assert_eq!(grid.count(RPSCell::Paper), 13);
    }

    #[test]
    fn grid_neighbors_wrap() {
        let mut grid = SpatialGrid::new(3, 3);
        // Top-left corner (0,0): left-wrap=(2,0), right=(1,0), up-wrap=(0,2), down=(0,1)
        grid.set(2, 0, RPSCell::Rock);
        grid.set(1, 0, RPSCell::Scissors);
        grid.set(0, 2, RPSCell::Rock);
        grid.set(0, 1, RPSCell::Scissors);

        let neighbors = grid.neighbors(0, 0);
        assert_eq!(neighbors.len(), 4);
        assert!(neighbors.contains(&RPSCell::Rock));
        assert!(neighbors.contains(&RPSCell::Scissors));
    }

    #[test]
    fn grid_step_invasion() {
        // 3x3 grid all Rock, center cell is Scissors.
        // Rock beats Scissors, so center should become Rock after one step.
        let mut grid = SpatialGrid::new(3, 3);
        for y in 0..3 {
            for x in 0..3 {
                grid.set(x, y, RPSCell::Rock);
            }
        }
        grid.set(1, 1, RPSCell::Scissors);
        grid.step();
        assert_eq!(grid.get(1, 1), RPSCell::Rock);
    }

    #[test]
    fn spiral_wave_dominant() {
        let wave = SpiralWave {
            generation: 1,
            rock_count: 80,
            paper_count: 10,
            scissors_count: 10,
        };
        assert_eq!(wave.total_cells(), 100);
        assert_eq!(wave.dominant_species(), Some(RPSCell::Rock));
    }

    #[test]
    fn spiral_wave_coexistence() {
        let wave = SpiralWave {
            generation: 1,
            rock_count: 34,
            paper_count: 33,
            scissors_count: 33,
        };
        assert!(wave.is_coexisting());

        // Below 5% threshold: rock_count=1 out of 100 total
        let wave2 = SpiralWave {
            generation: 2,
            rock_count: 1,
            paper_count: 50,
            scissors_count: 49,
        };
        assert!(!wave2.is_coexisting());
    }

    #[test]
    fn biodiversity_uniform() {
        // All-Paper grid should have entropy ~0
        let grid = SpatialGrid::new(10, 10);
        let idx = BiodiversityIndex::compute(&grid);
        assert!(
            idx.shannon_entropy < 1e-9,
            "expected ~0 entropy, got {}",
            idx.shannon_entropy
        );
        assert!(idx.simpson_index < 1e-9);
    }

    #[test]
    fn biodiversity_equal() {
        // Build a 3x9 grid with exactly 9 of each species (equal thirds of 27)
        let mut grid = SpatialGrid::new(9, 3);
        for x in 0..9 {
            grid.set(x, 0, RPSCell::Rock);
            grid.set(x, 1, RPSCell::Scissors);
            // row 2 remains Paper
        }
        let idx = BiodiversityIndex::compute(&grid);
        let ln3 = 3.0f64.ln();
        assert!(
            idx.shannon_entropy > 0.9 * ln3,
            "expected high entropy, got {}",
            idx.shannon_entropy
        );
        assert!(idx.is_diverse());
    }

    #[test]
    fn invasion_fronts_detected() {
        // Rock at (0,0), everything else Paper. Paper beats Rock -> front at (0,0).
        let mut grid = SpatialGrid::new(5, 5);
        grid.set(0, 0, RPSCell::Rock);
        let fronts = detect_invasion_fronts(&grid);
        assert!(!fronts.is_empty(), "expected invasion fronts");
        assert!(
            fronts
                .iter()
                .any(|&(x, y, attacker, _)| x == 0 && y == 0 && attacker == RPSCell::Paper),
            "expected Paper attacking Rock at (0,0)"
        );
    }

    #[test]
    fn coexistence_metric_all() {
        let history: Vec<SpiralWave> = (0..10)
            .map(|i| SpiralWave {
                generation: i,
                rock_count: 34,
                paper_count: 33,
                scissors_count: 33,
            })
            .collect();
        let metric = coexistence_metric(&history);
        assert!((metric - 1.0).abs() < 1e-9, "expected 1.0, got {}", metric);
    }

    #[test]
    fn run_simulation_length() {
        let snapshots = run_simulation(10, 10, 5, 42);
        assert_eq!(snapshots.len(), 5, "expected 5 snapshots");
        for (i, snap) in snapshots.iter().enumerate() {
            assert_eq!(snap.generation, (i + 1) as u64);
        }
    }

    #[test]
    fn rps_to_trit_roundtrip() {
        // to_trit is the inverse of from_trit for every valid cell.
        for cell in [RPSCell::Rock, RPSCell::Paper, RPSCell::Scissors] {
            assert_eq!(RPSCell::from_trit(cell.to_trit()), Some(cell));
        }
        assert_eq!(RPSCell::Rock.to_trit(), -1);
        assert_eq!(RPSCell::Paper.to_trit(), 0);
        assert_eq!(RPSCell::Scissors.to_trit(), 1);
    }

    #[test]
    fn rps_from_trit_out_of_range() {
        // Every i8 outside {-1, 0, 1} is invalid.
        for t in [-128, -2, 2, 3, 127] {
            assert_eq!(RPSCell::from_trit(t), None, "trit {t} should be invalid");
        }
    }

    #[test]
    fn grid_empty_is_noop() {
        // A 0x0 grid has no cells; every operation is a harmless no-op.
        let mut grid = SpatialGrid::new(0, 0);
        assert_eq!(grid.count(RPSCell::Paper), 0);
        grid.step(); // must not panic
        let idx = BiodiversityIndex::compute(&grid);
        assert_eq!(idx.shannon_entropy, 0.0);
        assert_eq!(idx.simpson_index, 0.0);
        assert_eq!(idx.evenness, 0.0);
        assert!(!idx.is_diverse());
    }

    #[test]
    fn grid_single_cell_is_stable() {
        // On a 1x1 torus the sole cell is all four of its own neighbors; since
        // no species beats itself, the cell never changes.
        let mut grid = SpatialGrid::new(1, 1);
        grid.set(0, 0, RPSCell::Rock);
        assert_eq!(grid.neighbors(0, 0), vec![RPSCell::Rock; 4]);
        for _ in 0..5 {
            grid.step();
            assert_eq!(grid.get(0, 0), RPSCell::Rock);
        }
    }

    #[test]
    fn dominant_species_none_when_balanced() {
        // No species exceeds 50% -> None.
        let wave = SpiralWave {
            generation: 1,
            rock_count: 4,
            paper_count: 4,
            scissors_count: 4,
        };
        assert_eq!(wave.total_cells(), 12);
        assert_eq!(wave.dominant_species(), None);
    }

    #[test]
    fn dominant_species_empty_is_none() {
        let wave = SpiralWave {
            generation: 0,
            rock_count: 0,
            paper_count: 0,
            scissors_count: 0,
        };
        assert_eq!(wave.total_cells(), 0);
        assert_eq!(wave.dominant_species(), None);
        assert!(!wave.is_coexisting());
    }

    #[test]
    fn biodiversity_hand_derived() {
        // Independent hand derivation for Rock=2, Paper=1, Scissors=0 (total 3):
        //   p        = (2/3, 1/3, 0)
        //   Shannon  = -(2/3 ln(2/3) + 1/3 ln(1/3)) = 0.6365141682948128
        //   Simpson  = 1 - ((2/3)^2 + (1/3)^2)      = 4/9 = 0.4444444444444444
        //   Evenness = Shannon / ln(3)              = 0.5793801642856949
        let mut grid = SpatialGrid::new(3, 1); // 3 Paper by default
        grid.set(0, 0, RPSCell::Rock);
        grid.set(1, 0, RPSCell::Rock);
        let idx = BiodiversityIndex::compute(&grid);
        assert!(
            (idx.shannon_entropy - 0.6365141682948128).abs() < 1e-12,
            "shannon {}",
            idx.shannon_entropy
        );
        assert!(
            (idx.simpson_index - 4.0 / 9.0).abs() < 1e-12,
            "simpson {}",
            idx.simpson_index
        );
        assert!(
            (idx.evenness - 0.5793801642856949).abs() < 1e-12,
            "evenness {}",
            idx.evenness
        );
        assert!(idx.is_diverse());
    }

    #[test]
    fn biodiversity_from_wave_matches_compute() {
        // from_wave must agree with compute for the same counts.
        let mut grid = SpatialGrid::new(4, 4);
        grid.set(0, 0, RPSCell::Rock);
        grid.set(1, 0, RPSCell::Scissors);
        let wave = SpiralWave::from_grid(&grid, 1);
        let a = BiodiversityIndex::compute(&grid);
        let b = BiodiversityIndex::from_wave(&wave);
        assert!((a.shannon_entropy - b.shannon_entropy).abs() < 1e-12);
        assert!((a.simpson_index - b.simpson_index).abs() < 1e-12);
        assert!((a.evenness - b.evenness).abs() < 1e-12);
    }

    #[test]
    fn coexistence_metric_empty_and_partial() {
        // Empty history -> 0.0 (no division by zero).
        assert_eq!(coexistence_metric(&[]), 0.0);
        // Mixed history: 2 of 3 generations coexist -> 2/3.
        let co = || SpiralWave {
            generation: 0,
            rock_count: 34,
            paper_count: 33,
            scissors_count: 33,
        };
        let not = SpiralWave {
            generation: 0,
            rock_count: 1,
            paper_count: 50,
            scissors_count: 49,
        };
        assert!((coexistence_metric(&[co(), co(), not]) - 2.0 / 3.0).abs() < 1e-12);
    }
}
