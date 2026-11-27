"""Standard-library stdio transport and file I/O for the Mojo driver."""
import json
import os
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent
MAX_FRAME = 8 * 1024 * 1024


def initialize_params():
    return {
        "protocolVersion": "2025-11-25",
        "capabilities": {},
        "clientInfo": {"name": "mojo-flight-energy", "version": "0.1.0"},
    }


def load_call():
    return json.loads((ROOT / "fixtures/six-phase.json").read_text(encoding="utf-8"))


class Session:
    """A local executable, one ordered request at a time, stderr inherited."""

    def __init__(self, executable):
        self.process = subprocess.Popen(
            [str(Path(executable).resolve())], stdin=subprocess.PIPE, stdout=subprocess.PIPE
        )
        self.reader = ThreadPoolExecutor(max_workers=1)
        self.last_request = {}
        self.next_id = 1

    def _send(self, frame):
        self.process.stdin.write(json.dumps(frame, allow_nan=False).encode("utf-8") + b"\n")
        self.process.stdin.flush()

    def notify(self, method):
        self._send({"jsonrpc": "2.0", "method": method})

    def request(self, method, params=None):
        frame = {"jsonrpc": "2.0", "id": self.next_id, "method": method}
        if params is not None:
            frame["params"] = params
        self.last_request = frame
        self.next_id += 1
        self._send(frame)
        return self.reader.submit(self._read_response, frame["id"]).result(timeout=15)

    def _read_response(self, expected_id):
        while True:
            line = self.process.stdout.readline(MAX_FRAME + 1)
            if len(line) > MAX_FRAME or not line.endswith(b"\n"):
                raise RuntimeError("server closed stdout or returned an oversized/incomplete frame")
            frame = json.loads(line)
            if not isinstance(frame, dict) or frame.get("jsonrpc") != "2.0":
                raise ValueError("expected a JSON-RPC 2.0 frame")
            if "id" not in frame and isinstance(frame.get("method"), str):
                continue  # Unsolicited MCP notification.
            if type(frame.get("id")) is not int or frame["id"] != expected_id:
                raise ValueError(f"unexpected JSON-RPC response id for request {expected_id}")
            if "error" in frame:
                raise RuntimeError(f"JSON-RPC error: {frame['error']}")
            return frame["result"]

    def finish(self):
        self.process.stdin.close()
        status = self.process.wait(timeout=5)
        if status:
            raise RuntimeError(f"server exited with status {status}")
        self.process.stdout.close()
        self.reader.shutdown()

    def abort(self):
        self.process.kill()
        self.process.wait()
        self.reader.shutdown()
        self.process.stdin.close()
        self.process.stdout.close()


def summary(result):
    totals = result["totals"]
    return (
        f"Required {totals['required_wh']:g} Wh | "
        f"projected remaining {totals['projected_remaining_wh']:g} Wh | "
        f"reserve margin {totals['reserve_margin_wh']:+g} Wh"
    )


def save_context(request, result):
    """Replace the previous handoff only after the driver validates the response."""
    sources = "\n".join(f"- {s['profile_id']} — {s['source']}" for s in result["segments"])
    blocks = [
        "# Flight energy demo context",
        summary(result),
        "## Tool request",
        "```json\n" + json.dumps(request, indent=2, allow_nan=False) + "\n```",
        "## Structured result",
        "```json\n" + json.dumps(result, indent=2, allow_nan=False) + "\n```",
        "## Selected profile sources",
        sources,
        "## Reproduce",
        "From the repository root run `pixi run --locked demo`. "
        "The input is [fixtures/six-phase.json](fixtures/six-phase.json).",
        "## Known limitations",
        "The six phases use synthetic measurements and caller-supplied durations. "
        "The independent [Matrice 100 benchmark](../drone-energy-planner/benchmark/README.md) covers one recorded configuration. "
        "The result measures energy reserve. The harness runs no native model inference.",
    ]
    destination = ROOT / "CONTEXT.md"
    with tempfile.NamedTemporaryFile(dir=ROOT, mode="w", encoding="utf-8", delete=False) as out:
        staged = Path(out.name)
        try:
            out.write("\n\n".join(blocks) + "\n")
            out.flush()
            os.fsync(out.fileno())
            os.replace(staged, destination)
        finally:
            staged.unlink(missing_ok=True)
