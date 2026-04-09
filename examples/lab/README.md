# Saddle Rendering Stochastic Texturing Lab

Crate-local lab for `saddle-rendering-stochastic-texturing`.

Run the interactive lab:

```bash
cargo run -p saddle-rendering-stochastic-texturing-lab
```

Run a scaffold E2E scenario:

```bash
cargo run -p saddle-rendering-stochastic-texturing-lab --features e2e -- smoke_launch
```

Discoverable scenario names are exposed through:

- `list_scenarios()`
- `scenario_by_name()`

The current lab is intentionally a scaffold. It validates structure, diagnostics, and authoring coverage while the production shader backend is still being implemented.

