# PLUG_AND_PLAY — Spiral

> Spiral optimization algorithms using ternary search landscapes

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ternary-spiral = { git = "https://github.com/SuperInstance/ternary-spiral" }
```

Use in your code:

```rust
use ternary_spiral::SpiralOptimizer;

let mut opt = SpiralOptimizer::new(-10.0, 10.0);
let best = opt.optimize(|x| x.sin());
```

## 📚 Available Documentation

| Document | Description |
|----------|-------------|
| `docs/FROM_BINARY.md` | Understanding ternary concepts as a binary programmer |
| `docs/MIGRATION.md` | Version migration guide |
| `docs/FUTURE-INTEGRATION.md` | Planned features and roadmap |

## 🔗 Integration

This crate is part of the [SuperInstance ternary fleet](https://github.com/SuperInstance). It uses the canonical `Ternary` type from `ternary-types` for cross-crate compatibility.

## 📄 License

MIT
