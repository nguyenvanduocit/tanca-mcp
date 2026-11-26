//! MCP tool definitions and the MCP server handler.

mod employee;
mod request;
mod shift;
mod shop;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
    ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};

/// Server name reported during `initialize`.
pub const SERVER_NAME: &str = "Tanca MCP";
/// Server version reported during `initialize`.
pub const SERVER_VERSION: &str = "1.0.0";

/// Why a tool call failed.
#[derive(Debug)]
pub enum ToolError {
    /// The arguments did not match the input type. Returned to the caller
    /// as a tool result with `isError: true`.
    Bind(String),
    /// The tool ran and failed. Returned as a JSON-RPC internal error.
    Failed(String),
}

type ToolFuture = Pin<Box<dyn Future<Output = Result<String, ToolError>> + Send>>;
type Handler = fn(Value) -> ToolFuture;

/// A registered tool: its definition and handler.
#[derive(Clone)]
pub struct ToolEntry {
    pub tool: Tool,
    handler: Handler,
}

/// JSON type of a tool parameter.
#[derive(Clone, Copy)]
pub(crate) enum ParamType {
    String,
    Number,
}

/// One input parameter of a tool.
pub(crate) struct Param {
    pub name: &'static str,
    pub kind: ParamType,
    pub required: bool,
    pub description: &'static str,
}

pub(crate) const fn string(name: &'static str, description: &'static str) -> Param {
    Param {
        name,
        kind: ParamType::String,
        required: false,
        description,
    }
}

pub(crate) const fn required_string(name: &'static str, description: &'static str) -> Param {
    Param {
        name,
        kind: ParamType::String,
        required: true,
        description,
    }
}

pub(crate) const fn number(name: &'static str, description: &'static str) -> Param {
    Param {
        name,
        kind: ParamType::Number,
        required: false,
        description,
    }
}

/// Builds a tool entry. The schema and annotations match the Go server.
pub(crate) fn entry(
    name: &'static str,
    description: &'static str,
    params: &[Param],
    handler: Handler,
) -> ToolEntry {
    let mut schema = Map::new();
    schema.insert("type".into(), json!("object"));
    if !params.is_empty() {
        let mut props = Map::new();
        for p in params {
            let ty = match p.kind {
                ParamType::String => "string",
                ParamType::Number => "number",
            };
            props.insert(
                p.name.into(),
                json!({ "type": ty, "description": p.description }),
            );
        }
        schema.insert("properties".into(), Value::Object(props));
    }
    let required: Vec<&str> = params
        .iter()
        .filter(|p| p.required)
        .map(|p| p.name)
        .collect();
    if !required.is_empty() {
        schema.insert("required".into(), json!(required));
    }

    let annotations = ToolAnnotations::new()
        .read_only(false)
        .destructive(true)
        .idempotent(false)
        .open_world(true);
    let tool = Tool::new(name, description, Arc::new(schema)).annotate(annotations);
    ToolEntry { tool, handler }
}

/// Decodes tool arguments, treating `null` as an absent value.
pub(crate) fn bind<T: DeserializeOwned>(args: Value) -> Result<T, ToolError> {
    crate::json::from_value(args)
        .map_err(|e| ToolError::Bind(format!("failed to bind arguments: {e}")))
}

/// Maps a client error to a tool failure with a prefix.
pub(crate) fn failed(prefix: &str, err: anyhow::Error) -> ToolError {
    ToolError::Failed(format!("{prefix}: {err:#}"))
}

/// All tools, sorted by name.
pub fn all_tools() -> Vec<ToolEntry> {
    let mut tools = Vec::new();
    tools.extend(shop::tools());
    tools.extend(employee::tools());
    tools.extend(shift::tools());
    tools.extend(request::tools());
    tools.sort_by(|a, b| a.tool.name.cmp(&b.tool.name));
    tools
}

/// Runs a tool by name. Returns `None` for an unknown tool.
pub async fn call_tool(
    tools: &[ToolEntry],
    name: &str,
    args: Option<Map<String, Value>>,
) -> Option<Result<String, ToolError>> {
    let entry = tools.iter().find(|t| t.tool.name == name)?;
    let args = Value::Object(args.unwrap_or_default());
    Some((entry.handler)(args).await)
}

/// The Tanca MCP server.
#[derive(Clone)]
pub struct TancaServer {
    tools: Arc<Vec<ToolEntry>>,
}

impl Default for TancaServer {
    fn default() -> Self {
        Self::new()
    }
}

impl TancaServer {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(all_tools()),
        }
    }
}

impl ServerHandler for TancaServer {
    #[allow(deprecated)] // logging capability kept for parity with the Go server
    fn get_info(&self) -> ServerConfig {
        let capabilities = ServerCapabilities::builder()
            .enable_logging()
            .enable_tools()
            .enable_tool_list_changed()
            .build();
        ServerConfig::new(capabilities)
            .with_server_info(Implementation::new(SERVER_NAME, SERVER_VERSION))
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(
            self.tools.iter().map(|t| t.tool.clone()).collect(),
        ))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let name = request.name.to_string();
        match call_tool(&self.tools, &name, request.arguments).await {
            None => Err(McpError::invalid_params(
                format!("tool '{name}' not found: tool not found"),
                None,
            )),
            Some(Ok(text)) => Ok(CallToolResult::success(vec![ContentBlock::text(text)]).into()),
            Some(Err(ToolError::Bind(msg))) => {
                Ok(CallToolResult::error(vec![ContentBlock::text(msg)]).into())
            }
            Some(Err(ToolError::Failed(msg))) => Err(McpError::internal_error(msg, None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_all_tools_sorted() {
        let names: Vec<String> = all_tools()
            .iter()
            .map(|t| t.tool.name.to_string())
            .collect();
        assert_eq!(
            names,
            [
                "tanca_check_in",
                "tanca_check_out",
                "tanca_get_clock_logs",
                "tanca_get_request_detail",
                "tanca_get_shift_by_day",
                "tanca_get_shift_summary",
                "tanca_get_shop_info",
                "tanca_get_today_shift",
                "tanca_list_employees",
                "tanca_list_requests",
                "tanca_list_shifts",
                "tanca_list_timekeeping_employees",
                "tanca_make_shift_change_request",
            ]
        );
    }

    #[tokio::test]
    async fn bad_arguments_are_bind_errors() {
        let tools = all_tools();
        let args = serde_json::from_value(json!({"page": "x"})).unwrap();
        let res = call_tool(&tools, "tanca_list_requests", Some(args))
            .await
            .unwrap();
        assert!(
            matches!(res, Err(ToolError::Bind(m)) if m.starts_with("failed to bind arguments: "))
        );
    }

    #[tokio::test]
    async fn invalid_time_fails_before_any_request() {
        let tools = all_tools();
        let args = serde_json::from_value(json!({"checkin_time": "bad"})).unwrap();
        let res = call_tool(&tools, "tanca_check_in", Some(args))
            .await
            .unwrap();
        assert!(
            matches!(res, Err(ToolError::Failed(m)) if m.starts_with("invalid checkin_time format (use YYYY-MM-DD HH:mm:ss): "))
        );
    }
}
