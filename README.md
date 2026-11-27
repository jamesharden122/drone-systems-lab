# Drone Systems Lab

The energy project’s goal is to add a read-only mission energy tool to the X500 Jetson companion’s general drone MCP server. Its first working component answers a planning question before flight: given the usable battery energy and a proposed sequence of flight phases, how much energy would remain at landing, and would that meet the reserve? The same Rust calculation runs in the desktop demo and is available to the companion.

In the broader X500 project, the companion owns onboard planning and PX4 owns flight control. This repository gives the planning code a home where it can be tested against recorded flights and synthetic missions without aircraft hardware. It also hosts a separate Mojo experiment exploring what inference could eventually run locally on the companion.

| Project | What runs today |
|---|---|
| [Drone energy planner](drone-energy-planner/) | Dependency-free Rust library that computes mission demand and signed reserve margin. The profile API reports the matched evidence per segment. |
| [Flight Energy MCP](flight-energy-mcp/README.md) | Read-only stdio server and Mojo client. The client estimates a synthetic six-phase mission and saves the exact request and result. The demo uses the shared planner crate in this repository. |
| [Energy Web Tools](energy_web_tools/README.md) | Dioxus application for distributed energy simulations. Its test is referenced on the [Tesero Sol renewable energy consulting page](https://tesorosol.com/renewable-energy-consulting). |
| [Mojo LLM](mojo-llm/README.md) | Native Mojo build and smoke test. The CPU-first inference experiment uses llama.cpp/ggml as a design reference. GGUF loading, quantized kernels, and token generation are future work. |

## Reproduce

Run each command from the repository root. Fetch the locked Rust packages with `cargo fetch --locked` if your cache is empty. The two Pixi projects preserve their individual locks. The MCP demo needs Pixi, Cargo, a C linker, and a host CPython with a shared library. See the demo README for the tested environment.

```sh
cargo test --workspace --locked --offline
(cd flight-energy-mcp && pixi run --locked demo)
(cd flight-energy-mcp && pixi run --locked test-demo)
(cd mojo-llm && pixi run --locked smoke)
```

The synthetic demo should report **46 Wh required**, **19 Wh projected remaining**, and a **+4 Wh reserve margin**. It writes the request and structured result to the [demo handoff](flight-energy-mcp/CONTEXT.md).

The [Matrice 100 benchmark](drone-energy-planner/benchmark/README.md) fits profiles on flights 167, 178, and 253, then checks two complete held-out flights with recorded phase durations. The predicted demand exceeded observed demand by 0.783 Wh for flight 276 and 0.598 Wh for flight 277. The reduced fixture includes its CC BY 4.0 attribution and reproduction instructions. These predictions apply to the recorded Matrice configuration.

## Companion integration

In the assembly workspace, `x500-px4-sim/companion` exposes the planner as `drone_agent_engine::energy_planner`. The planned onboard tool will accept a candidate mission through the companion’s existing MCP server, read the latest validated battery estimate from companion state, and call the planner in-process. The caller will receive mission totals and the supporting power profile for each phase. The standalone Flight Energy MCP demo calls the same crate directly and serves as a test harness for this integration. Running the demo exercises the calculation on a development machine without launching the companion or connecting to PX4.

For onboard use, the companion will need to supply validated usable battery energy, the reserve, candidate flight phases and power profiles for the X500 configuration. It can then use the signed reserve margin to compare plans or log a shortfall. Live telemetry integration and X500 profile calibration are still ahead. The Mojo inference experiment is independent of this calculation.

### Proposed battery telemetry through the flight controller

A flight controller could forward pack measurements to the Jetson using the conventions in the X500 Sensor Sim UDP guide. Its proposed `IMU1` and `HLT1` records cover motion and controller health. A dedicated battery record would reuse their source ID, controller session ID, sequence number and acquisition time, and carry pack voltage, signed current and measurement validity. The companion adapter would decode the record and publish a battery snapshot for the planner.

| Controller observation | How the companion would use it |
|---|---|
| Pack voltage (V) and current (A), sampled together | With discharge current defined as positive, voltage × current gives pack power in watts. Phase averages from recorded flights can supply evidence for future mission power profiles. |
| Acquisition time and pack power | Average successive valid power readings and multiply by elapsed seconds / 3600 to estimate consumed Wh. Subtract consumption from a validated starting usable-energy estimate for the same pack to obtain `remaining_usable_wh`. |
| Source, session, sequence and validity | Track each pack source independently. Ignore duplicates, flag gaps and stale samples, and end the integration interval at a controller restart. Resume planning when the remaining-energy estimate is valid again. |

The companion supplies the configured `reserve_wh` separately. For each candidate mission, the planner combines the latest usable-energy estimate with the expected power and duration of its phases, then returns projected remaining Wh and reserve margin. This telemetry path is proposed; the runnable demo supplies those inputs from a fixture.
