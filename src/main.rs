//! Tanca MCP server over stdio (default) or streamable HTTP (`--http_port`).

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::process::ExitCode;

use rmcp::ServiceExt;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use tanca::goflag::FlagSet;
use tanca::logf;
use tanca::tools::TancaServer;

#[tokio::main]
async fn main() -> ExitCode {
    let program = std::env::args()
        .next()
        .unwrap_or_else(|| "tanca-mcp".into());
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut fs = FlagSet::new(program);
    fs.string(
        "env",
        "",
        "Path to environment file (optional when environment variables are set directly)",
    )
    .string(
        "http_port",
        "",
        "Port for HTTP server. If not provided, will use stdio",
    );
    let flags = fs.parse_or_exit(&args);
    let env_file = flags.str("env");
    let http_port = flags.str("http_port");

    if !env_file.is_empty() {
        match dotenvy::from_path(&env_file) {
            Ok(()) => eprintln!("Loaded environment variables from {env_file}"),
            Err(e) => eprintln!("Warning: Error loading env file {env_file}: {e}"),
        }
    }

    if std::env::var("TANCA_TOKEN").unwrap_or_default().is_empty() {
        eprint!(
            "Configuration Error: Missing TANCA_TOKEN

Setup Instructions:
1. Get your Tanca API token from Tanca admin panel
2. Set the environment variable:

   Option A - Using .env file:
   TANCA_TOKEN=Bearer your-token-here
   TANCA_SHOP_ID=your-shop-id (optional, auto-fetched)
   TANCA_BRANCH_ID=your-branch-id (optional)

   Option B - Using environment variables:
   export TANCA_TOKEN=\"Bearer your-token-here\"

"
        );
        return ExitCode::from(1);
    }

    eprintln!("TANCA_TOKEN is set");
    match std::env::var("TANCA_SHOP_ID") {
        Ok(id) if !id.is_empty() => eprintln!("Shop ID: {id}"),
        _ => eprintln!("Shop ID: (will be auto-fetched)"),
    }

    let result = if http_port.is_empty() {
        serve_stdio().await
    } else {
        serve_http(&http_port).await
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            logf!("Server error: {e:#}");
            ExitCode::from(1)
        }
    }
}

async fn serve_stdio() -> anyhow::Result<()> {
    let service = TancaServer::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

async fn serve_http(port: &str) -> anyhow::Result<()> {
    println!();
    println!("Starting Tanca MCP Server in HTTP mode...");
    println!("Server will be available at: http://localhost:{port}/mcp");
    println!();
    println!("Claude Code Configuration:");
    println!("Add the following to your Claude Code MCP settings:");
    println!();
    println!("```json");
    println!("{{");
    println!("  \"mcpServers\": {{");
    println!("    \"tanca\": {{");
    println!("      \"url\": \"http://localhost:{port}/mcp\"");
    println!("    }}");
    println!("  }}");
    println!("}}");
    println!("```");
    println!();
    println!("Server starting...");

    let port: u16 = port
        .parse()
        .map_err(|_| anyhow::anyhow!("listen tcp: address {port}: invalid port"))?;

    // Like Go's ":port", accept connections on every interface and do not
    // restrict the Host header.
    let config = StreamableHttpServerConfig::default().disable_allowed_hosts();
    let service = StreamableHttpService::new(
        || Ok(TancaServer::new()),
        LocalSessionManager::default().into(),
        config,
    );
    let router = axum::Router::new().nest_service("/mcp", service);

    // Prefer a dual-stack IPv6 socket; fall back to IPv4 only.
    let listener = match tokio::net::TcpListener::bind(SocketAddr::from((
        Ipv6Addr::UNSPECIFIED,
        port,
    )))
    .await
    {
        Ok(l) => l,
        Err(_) => {
            tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port))).await?
        }
    };
    axum::serve(listener, router).await?;
    Ok(())
}
