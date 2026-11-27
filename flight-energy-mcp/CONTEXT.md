# Flight energy demo context

Required 46 Wh | projected remaining 19 Wh | reserve margin +4 Wh

## Tool request

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "estimate_profiled_mission",
    "arguments": {
      "remaining_usable_wh": 65,
      "reserve_wh": 15,
      "segments": [
        {
          "phase": "takeoff",
          "configuration": "synthetic-quad-A",
          "workload": "camera logging",
          "precipitation": "dry",
          "duration_s": 12,
          "conditions": [
            {
              "factor": "air_density_kg_m3",
              "value": 1.2
            }
          ]
        },
        {
          "phase": "climb",
          "configuration": "synthetic-quad-A",
          "workload": "camera logging",
          "precipitation": "dry",
          "duration_s": 48,
          "conditions": [
            {
              "factor": "air_density_kg_m3",
              "value": 1.2
            }
          ]
        },
        {
          "phase": "hover",
          "configuration": "synthetic-quad-A",
          "workload": "camera logging",
          "precipitation": "dry",
          "duration_s": 360,
          "conditions": [
            {
              "factor": "air_density_kg_m3",
              "value": 1.2
            }
          ]
        },
        {
          "phase": "cruise",
          "configuration": "synthetic-quad-A",
          "workload": "camera logging",
          "precipitation": "dry",
          "duration_s": 180,
          "conditions": [
            {
              "factor": "air_density_kg_m3",
              "value": 1.2
            }
          ]
        },
        {
          "phase": "descent",
          "configuration": "synthetic-quad-A",
          "workload": "camera logging",
          "precipitation": "dry",
          "duration_s": 48,
          "conditions": [
            {
              "factor": "air_density_kg_m3",
              "value": 1.2
            }
          ]
        },
        {
          "phase": "landing",
          "configuration": "synthetic-quad-A",
          "workload": "camera logging",
          "precipitation": "dry",
          "duration_s": 12,
          "conditions": [
            {
              "factor": "air_density_kg_m3",
              "value": 1.2
            }
          ]
        }
      ],
      "profiles": [
        {
          "id": "takeoff",
          "source": "planner/examples/six_phase.rs (synthetic)",
          "phase": "takeoff",
          "configuration": "synthetic-quad-A",
          "workload": "camera logging",
          "precipitation": "dry",
          "ranges": [
            {
              "factor": "air_density_kg_m3",
              "min": 1.18,
              "max": 1.22
            }
          ],
          "power": {
            "kind": "whole_pack",
            "watts": 450
          }
        },
        {
          "id": "climb",
          "source": "planner/examples/six_phase.rs (synthetic)",
          "phase": "climb",
          "configuration": "synthetic-quad-A",
          "workload": "camera logging",
          "precipitation": "dry",
          "ranges": [
            {
              "factor": "air_density_kg_m3",
              "min": 1.18,
              "max": 1.22
            }
          ],
          "power": {
            "kind": "whole_pack",
            "watts": 375
          }
        },
        {
          "id": "hover",
          "source": "planner/examples/six_phase.rs (synthetic)",
          "phase": "hover",
          "configuration": "synthetic-quad-A",
          "workload": "camera logging",
          "precipitation": "dry",
          "ranges": [
            {
              "factor": "air_density_kg_m3",
              "min": 1.18,
              "max": 1.22
            }
          ],
          "power": {
            "kind": "split",
            "propulsion": {
              "kind": "pack_side",
              "watts": 200
            },
            "compute": {
              "kind": "downstream",
              "watts": 9,
              "efficiency": 0.9
            },
            "auxiliary": {
              "kind": "pack_side",
              "watts": 10
            }
          }
        },
        {
          "id": "cruise",
          "source": "planner/examples/six_phase.rs (synthetic)",
          "phase": "cruise",
          "configuration": "synthetic-quad-A",
          "workload": "camera logging",
          "precipitation": "dry",
          "ranges": [
            {
              "factor": "air_density_kg_m3",
              "min": 1.18,
              "max": 1.22
            }
          ],
          "power": {
            "kind": "whole_pack",
            "watts": 280
          }
        },
        {
          "id": "descent",
          "source": "planner/examples/six_phase.rs (synthetic)",
          "phase": "descent",
          "configuration": "synthetic-quad-A",
          "workload": "camera logging",
          "precipitation": "dry",
          "ranges": [
            {
              "factor": "air_density_kg_m3",
              "min": 1.18,
              "max": 1.22
            }
          ],
          "power": {
            "kind": "whole_pack",
            "watts": 225
          }
        },
        {
          "id": "landing",
          "source": "planner/examples/six_phase.rs (synthetic)",
          "phase": "landing",
          "configuration": "synthetic-quad-A",
          "workload": "camera logging",
          "precipitation": "dry",
          "ranges": [
            {
              "factor": "air_density_kg_m3",
              "min": 1.18,
              "max": 1.22
            }
          ],
          "power": {
            "kind": "whole_pack",
            "watts": 150
          }
        }
      ]
    }
  }
}
```

## Structured result

```json
{
  "totals": {
    "required_wh": 46.0,
    "projected_remaining_wh": 19.0,
    "reserve_margin_wh": 4.0,
    "meets_reserve": true
  },
  "segments": [
    {
      "phase": "takeoff",
      "duration_s": 12.0,
      "profile_id": "takeoff",
      "source": "planner/examples/six_phase.rs (synthetic)",
      "pack_power_w": 450.0,
      "required_wh": 1.5
    },
    {
      "phase": "climb",
      "duration_s": 48.0,
      "profile_id": "climb",
      "source": "planner/examples/six_phase.rs (synthetic)",
      "pack_power_w": 375.0,
      "required_wh": 5.0
    },
    {
      "phase": "hover",
      "duration_s": 360.0,
      "profile_id": "hover",
      "source": "planner/examples/six_phase.rs (synthetic)",
      "pack_power_w": 220.0,
      "required_wh": 22.0
    },
    {
      "phase": "cruise",
      "duration_s": 180.0,
      "profile_id": "cruise",
      "source": "planner/examples/six_phase.rs (synthetic)",
      "pack_power_w": 280.0,
      "required_wh": 14.0
    },
    {
      "phase": "descent",
      "duration_s": 48.0,
      "profile_id": "descent",
      "source": "planner/examples/six_phase.rs (synthetic)",
      "pack_power_w": 225.0,
      "required_wh": 3.0
    },
    {
      "phase": "landing",
      "duration_s": 12.0,
      "profile_id": "landing",
      "source": "planner/examples/six_phase.rs (synthetic)",
      "pack_power_w": 150.0,
      "required_wh": 0.5
    }
  ]
}
```

## Selected profile sources

- takeoff — planner/examples/six_phase.rs (synthetic)
- climb — planner/examples/six_phase.rs (synthetic)
- hover — planner/examples/six_phase.rs (synthetic)
- cruise — planner/examples/six_phase.rs (synthetic)
- descent — planner/examples/six_phase.rs (synthetic)
- landing — planner/examples/six_phase.rs (synthetic)

## Reproduce

From the repository root run `pixi run --locked demo`. The input is [fixtures/six-phase.json](fixtures/six-phase.json).

## Known limitations

The six phases use synthetic measurements and caller-supplied durations. The independent [Matrice 100 benchmark](planner/benchmark/README.md) covers one recorded configuration. The result measures energy reserve. The harness runs no native model inference.
