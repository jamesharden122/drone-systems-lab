from std.python import Python, PythonObject
from std.sys import argv


def estimate(session: PythonObject, bridge: PythonObject) raises -> PythonObject:
    var tool = "estimate_profiled_mission"
    var hello = session.request("initialize", bridge.initialize_params())
    if String(py=hello["protocolVersion"]) != "2025-11-25":
        raise Error("server did not negotiate protocol 2025-11-25")
    _ = hello["capabilities"]["tools"]
    _ = session.notify("notifications/initialized")

    var catalog = session.request("tools/list")
    var tools = catalog["tools"]
    var found = False
    for index in range(len(tools)):
        var item = tools[index]
        if String(py=item["name"]) == tool:
            found = True
    if not found:
        raise Error("estimate_profiled_mission tool is missing")

    var call = bridge.load_call()
    var reply = session.request("tools/call", call)
    if Bool(py=reply.get("isError", False)):
        raise Error(String(py=reply["content"]))
    var result = reply["structuredContent"]
    var totals = result["totals"]
    if (
        Float64(py=totals["required_wh"]) != 46.0
        or Float64(py=totals["projected_remaining_wh"]) != 19.0
        or Float64(py=totals["reserve_margin_wh"]) != 4.0
        or not Bool(py=totals["meets_reserve"])
    ):
        raise Error("server totals disagree with the synthetic six-phase fixture")

    var rows = result["segments"]
    var profiles = call["arguments"]["profiles"]
    if len(rows) != 6:
        raise Error("expected evidence for all six phases")
    for index in range(len(rows)):
        var row = rows[index]
        var expected = profiles[index]
        if (
            String(py=row["phase"]) != String(py=expected["phase"])
            or String(py=row["profile_id"]) != String(py=expected["id"])
            or String(py=row["source"]) != String(py=expected["source"])
        ):
            raise Error("segment evidence does not match the fixture")
    return result


def main() raises:
    var args = argv()
    var server = String("target/debug/energy-mcp")
    if len(args) == 3 and args[1] == "--server":
        server = args[2]
    elif len(args) != 1:
        raise Error("usage: flight-energy-demo [--server EXECUTABLE]")

    Python.add_to_path("mojo")
    var bridge = Python.import_module("bridge")
    var session = bridge.Session(server)
    var result: PythonObject
    try:
        result = estimate(session, bridge)
        _ = session.finish()
    except error:
        _ = session.abort()
        raise error
    _ = bridge.save_context(session.last_request, result)
    print(bridge.summary(result))
    print("Saved CONTEXT.md")
