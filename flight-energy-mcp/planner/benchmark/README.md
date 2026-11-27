# Matrice 100 flight energy benchmark

Five flights on CMU AirLab’s instrumented DJI Matrice 100 flew route R1 with a 500 g payload, a programmed cruise altitude of 25 m and a programmed cruise ground speed of 10 m/s. This was the most frequently repeated combination of route, payload, altitude and speed in the published parameters file. We used all five recordings. Calibration uses flights 167, 178 and 253 from July and August 2019. Evaluation uses both complete flights from October 24, 2019, IDs 276 and 277. The split uses capture date, independent of energy or prediction error. [Dataset and parameters](https://doi.org/10.1184/R1/12683453.v1)

## Results

For each phase, the calibration estimates mean whole-pack watts as total phase Wh across the three training flights divided by total training hours in that phase. These means are 648.30 W during takeoff, 567.86 W during cruise and 523.09 W during landing. Each prediction multiplies the corresponding mean by the held-out phase duration. Signed error is predicted minus observed Wh.

| Held-out flight | Phase | Observed Wh | Predicted Wh | Signed error Wh |
|---|---|---:|---:|---:|
| 276 | Takeoff | 2.47784 | 2.68324 | +0.20541 |
| 276 | Cruise | 8.42669 | 8.97531 | +0.54861 |
| 276 | Landing | 3.93793 | 3.96676 | +0.02884 |
| **276** | **Airborne total** | **14.84246** | **15.62532** | **+0.78286** |
| 277 | Takeoff | 1.78812 | 1.96291 | +0.17479 |
| 277 | Cruise | 9.40612 | 9.82867 | +0.42255 |
| 277 | Landing | 3.93584 | 3.93625 | +0.00041 |
| **277** | **Airborne total** | **15.13007** | **15.72783** | **+0.59776** |

The comparison uses measured phase durations, so it evaluates power estimation conditional on knowing flight time. The small sample covers the recorded M100 configuration and route. Weather, CG, usable pack capacity and compute workload changes were not calibrated. The test labels precipitation as unreported and identifies equipment and onboard workload by dataset configuration. X500 predictions require separate calibration. [AirLab experiment description](https://theairlab.org/energy-dataset/)

## Time windows and integration

Use one synchronized CSV per flight. Each slice starts at the takeoff boundary and ends at the landing boundary, excluding preflight and postflight idle. Takeoff includes the climb to cruise. Landing includes descent. The [phase fixture](matrice100-phases.csv) records the exact boundaries in seconds since the flight file began.

Following the authors’ altitude-filter method, reflect the altitude samples at the endpoints, smooth with a Gaussian of standard deviation five samples (radius 20), and difference successive smoothed heights. The first rise greater than 0.05 m per sample marks takeoff onset. After the largest positive increment, the first nonpositive increment marks cruise onset. Around the most negative increment, the preceding and following nonnegative increments delimit landing. The reducer fixes these cut points and includes the sample at each boundary in both adjacent trapezoids to account for all elapsed time. [Authors’ phase method](https://github.com/thiago-a-rod/energy_consumption/blob/3058a25a7092633e05a3fe169c6ae3b967636b9d/FindingRegimeFilter.py) · [Published three-regime analysis](https://pmc.ncbi.nlm.nih.gov/articles/PMC9403403/)

At each recorded instant multiply CSV system voltage (V) by CSV system current (A), measured at the battery. The processed CSV uses positive current during airborne discharge. Integrate consecutive power samples with the trapezoid rule and their recorded time differences, then divide watt-seconds by 3,600. The largest interval in the five airborne windows is 1.30 s on flight 277. The trapezoids assume linear power between samples. A shared boundary sample ends one interval and starts the next. [Dataset instrumentation](https://pmc.ncbi.nlm.nih.gov/articles/PMC8213793/)

## Reproduction

The test runs offline using the 15-row phase fixture. To independently reproduce all observed, predicted and error columns, use Python 3.10 or later from the bundled planner directory (`cd planner` from the workspace root).

```sh
curl --fail --location https://ndownloader.figshare.com/files/26385070 --output /tmp/cmu-m100-flights.zip
python3 benchmark/reduce_cmu.py /tmp/cmu-m100-flights.zip > /tmp/cmu-m100-phases.csv
diff -u benchmark/matrice100-phases.csv /tmp/cmu-m100-phases.csv
cargo test --locked --offline --test real_flights
```

The reducer verifies the pinned source archive’s SHA-256. It reads only the five specified CSV members. The Rust test fits the three phase profiles solely from the calibration rows and checks the public planner output against the Python reference for both held-out flights.

The reduced CSV derives from *Data Collected with Package Delivery Quadcopter Drone*, version 1, by Thiago A. Rodrigues, Jay Patrikar, Arnav Choudhry, Jacob Feldgoise, Vaibhav Arcot, Aradhana Gahlaut, Sophia Lau, Brady Moon, Bastian Wagner, H. Scott Matthews, Sebastian Scherer and Constantine Samaras. Modifications are flight selection, segmentation, numerical integration, prediction and rounding. The reduced data retain the source’s Creative Commons Attribution 4.0 license. [Source data](https://doi.org/10.1184/R1/12683453.v1) · [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/)
