//! The only side effects in the server are reading and writing MCP frames.
mod wire;

use mcport::{McpServer, PreparedTool, ToolAnnotations, ToolDefinition, ToolReply};
use schemars::schema_for;
use std::io;
use wire::{EstimateRequest, EstimateResponse};

const TOOL: &str = "estimate_profiled_mission";

fn estimate(arguments: &str) -> ToolReply {
    let request: EstimateRequest = match serde_json::from_str(arguments) {
        Ok(request) => request,
        Err(error) => return ToolReply::error(format!("invalid tool arguments: {error}")),
    };
    match request.evaluate() {
        Ok(response) => ToolReply::structured(response),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

fn main() -> io::Result<()> {
    let mut definition = ToolDefinition::new(
        TOOL,
        "Estimate one battery budget using measured phase profiles. Returns signed reserve margin and profile evidence for each segment.",
        serde_json::from_value(schema_for!(EstimateRequest).to_value())?,
    );
    definition.output_schema = Some(serde_json::from_value(
        schema_for!(EstimateResponse).to_value(),
    )?);
    definition.annotations = ToolAnnotations {
        read_only_hint: Some(true),
        idempotent_hint: Some(true),
        destructive_hint: Some(false),
        open_world_hint: Some(false),
    };
    McpServer::new("energy-mcp", env!("CARGO_PKG_VERSION"))
        .strict_schemas()
        .tools([PreparedTool::stateless(definition, estimate)])
        .serve()
}
