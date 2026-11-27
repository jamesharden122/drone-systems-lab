"""Rebuild the Matrice 100 benchmark from the upstream v1 flights.zip (Python 3.10+)."""

import csv
import hashlib
import io
import math
from pathlib import Path
import sys
import zipfile

# Time since each file began, in seconds. Cut points are shared by adjacent
# phases so the trapezoid from t[i] to t[i+1] belongs to exactly one phase.
# See README.md for the altitude-based boundary extraction and group selection.
CALIBRATION = {
    167: (22.01, 35.20, 97.20, 120.60),
    178: (19.90, 32.20, 81.00, 117.70),
    253: (24.70, 37.10, 96.52, 123.40),
}
HELD_OUT = {
    276: (19.30, 34.20, 91.10, 118.40),
    277: (10.80, 21.70, 84.01, 111.10),
}
PHASES = ("takeoff", "cruise", "landing")
ARCHIVE_SHA256 = "98f65a1ab54a8b618c169fdf49311a53ff1fefed86fdc3f440d6a0166ce49fb1"


def observe(archive, flight, boundaries):
    """Integrate the product of simultaneous CSV voltage/current samples."""
    with archive.open(f"flights/{flight}.csv") as member:
        reader = csv.DictReader(io.TextIOWrapper(member, encoding="utf-8", newline=""))
        samples = [(float(row["time"]), float(row["battery_voltage"]) * float(row["battery_current"]))
                   for row in reader]
    times = [time for time, _ in samples]
    for start, end in zip(boundaries, boundaries[1:]):
        left, right = times.index(start), times.index(end)
        intervals = []
        for i in range(left, right):
            t0, p0 = samples[i]
            t1, p1 = samples[i + 1]
            if t1 <= t0 or not all(math.isfinite(x) for x in (t0, p0, t1, p1)):
                raise ValueError(f"Invalid paired samples in flight {flight} at row {i}")
            intervals.append((t1 - t0) * (p0 + p1) / 2)
        yield start, end, math.fsum(intervals) / 3600


def main(path):
    if hashlib.sha256(path.read_bytes()).hexdigest() != ARCHIVE_SHA256:
        raise ValueError("Expected CMU v1 flights.zip, file ID 26385070")
    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(("flight", "split", "phase", "start_s", "end_s", "observed_wh",
                     "predicted_wh", "signed_error_wh"))
    energy = dict.fromkeys(PHASES, 0.0)
    seconds = dict.fromkeys(PHASES, 0.0)
    with zipfile.ZipFile(path) as archive:
        for split, flights in (("calibration", CALIBRATION), ("held_out", HELD_OUT)):
            for flight, boundaries in flights.items():
                measurements = observe(archive, flight, boundaries)
                for phase, (start, end, observed_wh) in zip(PHASES, measurements, strict=True):
                    duration = end - start
                    prediction = error = ""
                    if split == "calibration":
                        energy[phase] += observed_wh
                        seconds[phase] += duration
                    else:
                        predicted_wh = (energy[phase] / seconds[phase]) * duration
                        prediction = f"{predicted_wh:.9f}"
                        error = f"{predicted_wh - observed_wh:.9f}"
                    writer.writerow((flight, split, phase, f"{start:.2f}", f"{end:.2f}",
                                     f"{observed_wh:.9f}", prediction, error))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit("Usage: python3 benchmark/reduce_cmu.py path/to/flights.zip")
    main(Path(sys.argv[1]))
