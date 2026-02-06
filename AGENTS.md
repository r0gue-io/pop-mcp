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
- Use "chain" not "parachain" (except in template names like `r0gue-io/base-parachain`)

### Testing
- Unit tests inline with `#[cfg(test)]`
- Integration tests in `tests/tools/`
- Descriptive test names: `build_contract_rejects_missing_path`
- Plain asserts only: `assert!(x)` not `assert!(x, "msg")` - test names should be descriptive enough

### Commits
- Conventional prefixes: `fix:`, `feat:`, `refactor:`
- Imperative, ≤72 chars
- **Pre-commit (Rust):** Run `cargo +nightly fmt --all` and `cargo clippy --all-features --all-targets` before committing Rust changes.

### Scope & Ownership
- Before changing tests to fix upstream behavior, confirm the root cause and agree whether the fix belongs upstream or in this repo.

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


## Adding New Tools

See `ADD_TOOL_PLAN.md` for the workflow.
