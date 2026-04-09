# Saddle Rendering Stochastic Texturing Lab

Crate-local lab for `saddle-rendering-stochastic-texturing`.

Run the interactive lab:

```bash
cargo run -p saddle-rendering-stochastic-texturing-lab
```

Run the interactive lab with BRP/debug tooling enabled:

```bash
cargo run -p saddle-rendering-stochastic-texturing-lab --features dev
```

Run a scaffold E2E scenario:

```bash
cargo run -p saddle-rendering-stochastic-texturing-lab --features e2e -- smoke_launch
```

The `dev` feature is intended for native runs. Web/docs builds use the default feature set so the lab stays wasm-compatible.

Discoverable scenario names are exposed through:

- `list_scenarios()`
- `scenario_by_name()`

The current lab is intentionally a scaffold. It validates structure, diagnostics, and authoring coverage while the production shader backend is still being implemented.
