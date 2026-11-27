# Drone energy planning and native Mojo experiments

This repository brings together a battery-budget library, a runnable MCP demo, and an early experiment in native Mojo inference. The energy workflow estimates how many watt-hours a candidate drone mission needs and shows which power profiles support the result.

| Project | What runs today |
|---|---|
| [Drone energy planner](drone-energy-planner/) | Dependency-free Rust library that computes mission demand and signed reserve margin. The profile API reports the matched evidence per segment. |
| [Flight Energy MCP](flight-energy-mcp/README.md) | Read-only stdio server and Mojo client. The client estimates a synthetic six-phase mission and saves the exact request and result. The demo bundles its own copy of the planner. |
| [Mojo LLM](mojo-llm/README.md) | Native Mojo build and smoke test. The CPU-first inference experiment uses llama.cpp/ggml as a design reference. GGUF loading, quantized kernels, and token generation are future work. |

## Reproduce

Run each command from this repository's root. The two Pixi projects preserve their individual locks. The MCP demo needs Pixi, Cargo, a C linker, and a host CPython with a shared library. See the demo README for the tested environment.

```sh
(cd drone-energy-planner && cargo test --locked --offline)
(cd flight-energy-mcp && cargo test --workspace --locked)
(cd flight-energy-mcp && pixi run --locked demo)
(cd flight-energy-mcp && pixi run --locked test-demo)
(cd mojo-llm && pixi run --locked smoke)
```

The synthetic demo should report **46 Wh required**, **19 Wh projected remaining**, and a **+4 Wh reserve margin**. It writes the request and structured result to the [demo handoff](flight-energy-mcp/CONTEXT.md).

The [Matrice 100 benchmark](drone-energy-planner/benchmark/README.md) fits profiles on flights 167, 178, and 253, then checks two complete held-out flights with recorded phase durations. The predicted demand exceeded observed demand by 0.783 Wh for flight 276 and 0.598 Wh for flight 277. The reduced fixture includes its CC BY 4.0 attribution and reproduction instructions. These predictions apply to the recorded Matrice configuration.

## Companion integration

The companion imports the standalone planner as a Rust library. The MCP demo exercises the public API over stdio. The companion remains responsible for supplying validated battery and mission inputs when it exposes an onboard tool.
