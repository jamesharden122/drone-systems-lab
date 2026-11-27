"""Integration checks against the built server and native Mojo driver."""
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "mojo"))
import bridge

SERVER = ROOT / "target/debug/energy-mcp"
DRIVER = ROOT / "build/flight-energy-demo"


class StdioTests(unittest.TestCase):
    def setUp(self):
        self.session = bridge.Session(SERVER)
        self.addCleanup(self.session.abort)
        hello = self.session.request("initialize", bridge.initialize_params())
        self.assertEqual(hello["protocolVersion"], "2025-11-25")
        self.assertIn("tools", hello["capabilities"])
        self.session.notify("notifications/initialized")

    def test_catalog_and_errors_then_valid_call(self):
        [tool] = self.session.request("tools/list")["tools"]
        self.assertEqual(tool["name"], "estimate_profiled_mission")
        self.assertIs(tool["annotations"]["readOnlyHint"], True)
        self.assertEqual(set(tool["inputSchema"]["required"]), {
            "remaining_usable_wh", "reserve_wh", "segments", "profiles",
        })

        bad = bridge.load_call()
        del bad["arguments"]["remaining_usable_wh"]
        unmatched = bridge.load_call()
        unmatched["arguments"]["segments"][2]["workload"] = "uncalibrated"
        for params, expected in [(bad, "remaining_usable_wh"), (unmatched, "segments[2]")]:
            reply = self.session.request("tools/call", params)
            self.assertIs(reply["isError"], True)
            self.assertIn(expected, reply["content"][0]["text"])

        result = self.session.request("tools/call", bridge.load_call())["structuredContent"]
        self.assertEqual(result["totals"], {
            "required_wh": 46.0, "projected_remaining_wh": 19.0,
            "reserve_margin_wh": 4.0, "meets_reserve": True,
        })
        self.assertEqual([s["required_wh"] for s in result["segments"]], [1.5, 5, 22, 14, 3, .5])


class MojoHandoffTests(unittest.TestCase):
    def test_alternate_executable_context_and_failure_preservation(self):
        with tempfile.TemporaryDirectory() as temp:
            workspace = Path(temp)
            (workspace / "mojo").mkdir()
            (workspace / "fixtures").mkdir()
            shutil.copy(ROOT / "mojo/bridge.py", workspace / "mojo")
            fixture = workspace / "fixtures/six-phase.json"
            shutil.copy(ROOT / "fixtures/six-phase.json", fixture)
            # No target/debug/server under this cwd. The driver must use the override.
            command = [str(DRIVER), "--server", str(SERVER)]
            run = subprocess.run(command, cwd=workspace, capture_output=True, text=True, timeout=30)
            self.assertEqual(run.returncode, 0, run.stderr)
            context = workspace / "CONTEXT.md"
            saved = context.read_text(encoding="utf-8")
            request, result = map(json.loads, re.findall(r"```json\n(.*?)\n```", saved, re.S))
            self.assertEqual(request["method"], "tools/call")
            self.assertEqual(request["params"], json.loads(fixture.read_text(encoding="utf-8")))
            self.assertEqual(bridge.summary(result), "Required 46 Wh | projected remaining 19 Wh | reserve margin +4 Wh")
            self.assertIn(bridge.summary(result), run.stdout.splitlines())
            self.assertIn(bridge.summary(result), saved.splitlines())
            for row, profile in zip(result["segments"], request["params"]["arguments"]["profiles"], strict=True):
                self.assertEqual((row["profile_id"], row["source"]), (profile["id"], profile["source"]))
            self.assertNotIn(str(ROOT), saved)
            self.assertNotIn(str(workspace), saved)

            # This request succeeds on the server, but its result fails Mojo's fixture check.
            changed = request["params"]
            changed["arguments"]["reserve_wh"] = 30
            fixture.write_text(json.dumps(changed), encoding="utf-8")
            run = subprocess.run(command, cwd=workspace, capture_output=True, text=True, timeout=30)
            self.assertNotEqual(run.returncode, 0)
            self.assertIn("totals disagree", run.stderr)
            self.assertEqual(context.read_text(encoding="utf-8"), saved)
