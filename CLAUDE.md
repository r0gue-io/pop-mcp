# Pop-MCP

## Why Pop-MCP Exists

Pop-MCP is an **AI usability layer** on top of Pop CLI. It exists because:

1. **AIs can't use interactive CLIs** - Pop CLI has prompts, confirmations, and interactive modes that AI agents cannot navigate
2. **AIs need structured I/O** - MCP provides typed params and structured responses instead of parsing terminal output
3. **AIs need discoverable tools** - Tool descriptions help AI agents pick the right tool for the task
4. **AIs work in workflows** - Tools should compose naturally (create → build → deploy → call)

**Design principle:** Every tool should be usable by an AI without human intervention. If it requires interactive input, wallet browser extensions, or manual steps - it doesn't belong in pop-mcp.

---

## Repository Guidelines

### Project Structure
- `src/main.rs` - MCP server entry
- `src/server.rs` - Tool registration
- `src/executor.rs` - Pop CLI command runner
- `src/tools/` - Tool implementations
- `tests/tools/` - Integration tests (gated by `pop-e2e` feature)

### Commands
```bash
cargo +nightly fmt                        # Format
cargo clippy --all-features --all-targets # Lint
cargo test                                # Unit tests
cargo test --features pop-e2e             # Integration tests
```

### Style
- Rust 2021, rustfmt defaults
- `snake_case` files/functions, `PascalCase` types
- `anyhow::Context` on fallible paths
- No panics in tool code

### Testing
- Unit tests inline with `#[cfg(test)]`
- Integration tests in `tests/tools/`
- Descriptive test names: `build_contract_rejects_missing_path`

### Commits
- Conventional prefixes: `fix:`, `feat:`, `refactor:`
- Imperative, ≤72 chars

---

## Tool Implementation Patterns

### Params Struct
```rust
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]  // CRITICAL for Claude Code
pub struct ToolParams {
    #[schemars(description = "Human-readable description")]
    pub required_field: String,

    #[schemars(description = "Optional with default note")]
    pub optional_field: Option<String>,
}
```

### Tool Function
```rust
pub fn tool_name(
    executor: &PopExecutor,
    params: ToolParams,
) -> PopMcpResult<CallToolResult> {
    params.validate().map_err(PopMcpError::InvalidInput)?;
    let args = build_tool_args(&params);

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(output)),
        Err(e) => Ok(error_result(format!("Failed: {}", e))),
    }
}
```

### Registration (`server.rs`)
```rust
#[tool(description = "Short description for AI discovery")]
async fn tool_name(&self, Parameters(params): Parameters<ToolParams>) -> Result<CallToolResult, McpError> {
    tool_name(&self.executor, params).map_err(|e| McpError::internal_error(e.to_string(), None))
}
```

---

## DO / DON'T

### DO
- Use `#[schemars(extend("properties" = {}))]` on all param structs
- Use `-y` flag for non-interactive execution
- Return `Ok(error_result(...))` for CLI failures
- Validate params before execution
- Test both success and failure paths

### DON'T
- Use `Parameters<()>` - use empty struct instead
- Use `Option<Enum>` - breaks Claude Code schema
- Panic in tool functions
- Require interactive input
- Return `Err` for CLI errors (use `error_result`)

---

## Gotchas

- **Schema:** Must have `"type": "object"` and `"properties": {}` at top level
- **Pop CLI:** Some commands return exit 0 on logical errors - check output text
- **Lifetimes:** Store owned strings before building arg refs
- **Ports:** Use non-default ports (9945, 8546) in tests to avoid conflicts

---

## Existing Tools

| Tool | Command | Key Params |
|------|---------|------------|
| `create_contract` | `pop new contract` | name, template |
| `build_contract` | `pop build --path` | path, release |
| `test_contract` | `pop test --path` | path, e2e |
| `deploy_contract` | `pop up <path>` | path, constructor, args, suri, url |
| `call_contract` | `pop call contract` | path, contract, message, args |
| `up_ink_node` | `pop up ink-node` | ports |
| `clean_nodes` | `pop clean node --pid` | pids |
| `convert_address` | `pop convert address` | address |

---

## Future Tools

| Tool | Command | Purpose |
|------|---------|---------|
| `up_network` | `pop up paseo/kusama/...` | Connect to live networks |
| `new_chain` | `pop new chain` | Create parachains |
| `new_pallet` | `pop new pallet` | Create pallets |
| `build_spec` | `pop build spec` | Build chain specs |
| `call_chain` | `pop call chain` | Runtime calls |

---

## Adding New Tools

See `ADD_TOOL_PLAN.md` for the workflow.
