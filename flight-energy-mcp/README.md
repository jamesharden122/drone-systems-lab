# Flight Energy MCP

How much battery energy will a proposed flight use, and which measurements support the estimate? This project answers those questions with a Rust planner, a read-only MCP tool, and a native Mojo client. It includes a six-phase demonstration with known answers and a separate benchmark derived from recorded flights.

## Run the demo

From the repository root on Linux:

```sh
pixi run --locked demo
```

The command builds the Rust server and Mojo client, runs the tool over stdio, and prints:

```text
Required 46 Wh | projected remaining 19 Wh | reserve margin +4 Wh
Saved CONTEXT.md
```

Open [CONTEXT.md](CONTEXT.md) for the exact tool request, structured response, selected profile sources, and reproduction command. A second coding session can read it directly. The driver replaces the file atomically after checking the result and server exit status. A failed run preserves the previous handoff.

**Prerequisites.** Install Pixi, a Rust 2024 toolchain with Cargo, a system C linker, and a shared-library build of CPython 3.10 or later available through `/usr/bin/python3`. The first build needs access to the locked Cargo and Pixi packages or populated package caches. The harness retains the original Linux x86-64 and AArch64 Pixi lock and Mojo 1.1.0 pin. Its locked x86-64 Python executable lacks an embeddable libpython, so the tasks select the host CPython through `MOJO_PYTHON`. Development verification used Linux x86-64, Rust 1.95, Pixi 0.66 and host CPython 3.12. AArch64 hardware execution remains unverified.

## Two sources of evidence

**Six-phase demo.** The [fixture](fixtures/six-phase.json) supplies a 65 Wh usable battery budget and a 15 Wh reserve. Takeoff, climb, hover, cruise, descent and landing use 1.5, 5, 22, 14, 3 and 0.5 Wh respectively. These are synthetic calibration inputs. For the six-minute hover, propulsion draws 200 W, the compute branch draws 9 W downstream of a 90%-efficient converter, and auxiliaries draw 10 W at the pack. The resulting 220 W gives 22 Wh. The [Rust example](planner/examples/six_phase.rs) obtains the same result from a whole-pack measurement and demonstrates a change of operating conditions.

**Measured Matrice 100 benchmark.** Three comparable flights (167, 178, 253) set the mean pack watts for takeoff, cruise and landing. Two entire flights from a later day were held out. The reference integrates paired voltage and current over recorded flight intervals. Supplying the actual phase durations isolates error in the power estimate.

| Held-out flight | Observed Wh | Predicted Wh | Predicted minus observed Wh |
|---|---:|---:|---:|
| 276 | 14.84246 | 15.62532 | +0.78286 |
| 277 | 15.13007 | 15.72783 | +0.59776 |

These two results describe the recorded M100 configuration and route. Applying the model to an X500 requires its own calibration. The [benchmark notes](planner/benchmark/README.md) record phase boundaries, assumptions, source attribution under CC BY 4.0, and how to regenerate the 15-row fixture from the checksum-verified CMU archive. [Original dataset](https://doi.org/10.1184/R1/12683453.v1)

## How it works

1. The [Mojo driver](mojo/main.mojo) negotiates MCP version `2025-11-25`, sends `notifications/initialized`, discovers the tool with `tools/list`, and issues `tools/call`. It checks totals and per-phase evidence. A [Python standard-library bridge](mojo/bridge.py) handles subprocess pipes, newline-delimited JSON-RPC and file I/O. Native model inference is future work. [MCP lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle) · [Mojo Python interop](https://mojolang.org/docs/manual/python/python-from-mojo/)
2. The [Rust server](energy-mcp/src/main.rs) registers one tool, `estimate_profiled_mission`, using **mcport 0.6.1**. Serde, serde_json and Schemars handle the JSON boundary and published schemas. Tool arguments carry `remaining_usable_wh`, `reserve_wh`, `segments` and `profiles`. Success returns `structuredContent.totals` and ordered `structuredContent.segments`. Malformed arguments and planning failures produce MCP tool errors.
3. The [numerical library](planner/src/profile.rs) has **zero external crates**. A segment must match one profile by phase, configuration, workload, precipitation and the same set of numeric factors inside inclusive ranges. Missing coverage and overlapping matches produce indexed errors. Each profile supplies total pack watts or disjoint electrical loads allocated to the planned pack. The library integrates watts over seconds and retains signed energy shortfalls. The [calibration guide](planner/docs/flight-energy.md) covers weather, CG, rotor behavior, compute workloads, and battery temperature.

## Reproduce the checks

```sh
cargo test --workspace --locked
cargo run --locked -p drone-energy-planner --example six_phase
cargo test --locked -p drone-energy-planner --test real_flights
pixi run --locked test-demo
```

The Rust tests and reduced benchmark can run with `--offline` after the Cargo dependencies are cached. The integration suite launches the real server, checks the catalog, sends malformed and unsupported inputs, then verifies a successful request on the same connection. It also checks that the displayed totals and the saved handoff agree and that a failed fixture check preserves the file.

For a different MCP client, launch `target/debug/energy-mcp` as a stdio subprocess after `pixi run --locked build-server`. For client integration testing, point the Mojo demo at another compatible executable:

```sh
pixi run --locked demo --server ./path/to/compatible-server
```

The compatible server must implement the negotiated lifecycle and the tool's structured result for the bundled fixture. A successful run records the returned evidence in the local handoff.
