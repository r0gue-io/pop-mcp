# MCP Structured Output & Tool Result Improvements

This document explores how to improve tool result handling in pop-mcp by leveraging MCP's structured content feature.

## Problem Statement

Current tests assert on string content, which is fragile:

```rust
// build - we don't care about the text, just success/fail
assert!(output.contains("Build successful"));

// call - we DO care about the contract state value
assert!(text(&result)?.contains("false"));

// deploy - fragile address detection
assert!(output.contains("0x") || output.contains("5"));
```

## MCP Specification Support (2025-11-25)

The MCP spec provides two ways to return tool results:

### Unstructured Content (`content`)

```json
{
  "content": [{ "type": "text", "text": "Build successful!" }],
  "isError": false
}
```

### Structured Content (`structuredContent`)

```json
{
  "content": [{ "type": "text", "text": "{\"address\": \"0x...\"}" }],
  "structuredContent": { "address": "0x..." },
  "isError": false
}
```

### Output Schema

Tools can declare an `outputSchema` to validate structured results:

```json
{
  "name": "deploy_contract",
  "inputSchema": { ... },
  "outputSchema": {
    "type": "object",
    "properties": {
      "address": { "type": "string" }
    },
    "required": ["address"]
  }
}
```

## rmcp 0.8.4 Support

The rmcp crate fully supports structured output:

### CallToolResult

```rust
pub struct CallToolResult {
    pub content: Vec<Content>,
    pub structured_content: Option<Value>,
    pub is_error: Option<bool>,
    pub meta: Option<Meta>,
}

impl CallToolResult {
    pub fn success(content: Vec<Content>) -> Self;
    pub fn error(content: Vec<Content>) -> Self;
    pub fn structured(value: Value) -> Self;       // Sets both content + structured_content
    pub fn structured_error(value: Value) -> Self;
    pub fn into_typed<T: DeserializeOwned>(self) -> Result<T, serde_json::Error>;
}
```

### Tool with Output Schema

```rust
impl Tool {
    pub fn with_output_schema<T: JsonSchema + 'static>(mut self) -> Self;
}
```

### Json<T> Wrapper

rmcp provides a `Json<T>` wrapper that automatically serializes to structured content:

```rust
pub struct Json<T>(pub T);

impl<T: Serialize + JsonSchema + 'static> IntoCallToolResult for Json<T> {
    fn into_call_tool_result(self) -> Result<CallToolResult, ErrorData> {
        let value = serde_json::to_value(self.0)?;
        Ok(CallToolResult::structured(value))
    }
}
```

### IntoCallToolResult Trait

```rust
pub trait IntoCallToolResult {
    fn into_call_tool_result(self) -> Result<CallToolResult, ErrorData>;
}

// Blanket impl for Result<T, E> where both impl IntoContents
impl<T: IntoContents, E: IntoContents> IntoCallToolResult for Result<T, E> {
    fn into_call_tool_result(self) -> Result<CallToolResult, ErrorData> {
        match self {
            Ok(value) => Ok(CallToolResult::success(value.into_contents())),
            Err(error) => Ok(CallToolResult::error(error.into_contents())),
        }
    }
}
```

## Proposed Implementation

### Option 1: Structured Output Types (Recommended)

Define output structs for tools that return meaningful data:

```rust
// src/tools/build/contract.rs
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
pub struct BuildOutput {
    pub success: bool,
}

// For tools with no meaningful output
pub fn build_contract<E: CommandExecutor>(
    executor: &E,
    params: BuildContractParams,
) -> Result<(), String> {  // Simple Result, no data on success
    match executor.execute(&args) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Build failed: {}", e)),
    }
}
```

```rust
// src/tools/up/contract.rs
#[derive(Debug, Serialize, JsonSchema)]
pub struct DeployOutput {
    pub address: String,
    pub raw_output: String,
}

pub fn deploy_contract<E: CommandExecutor>(
    executor: &E,
    params: DeployContractParams,
) -> Result<DeployOutput, String> {
    match executor.execute(&args) {
        Ok(output) => {
            let address = parse_contract_address(&output)
                .ok_or("Failed to parse contract address")?;
            Ok(DeployOutput { address, raw_output: output })
        }
        Err(e) => Err(format!("Deployment failed: {}", e)),
    }
}
```

```rust
// src/tools/call/contract.rs
#[derive(Debug, Serialize, JsonSchema)]
pub struct CallOutput {
    pub value: String,        // The actual return value
    pub raw_output: String,   // Full CLI output for LLM
}

pub fn call_contract<E: CommandExecutor>(
    executor: &E,
    params: CallContractParams,
) -> Result<CallOutput, String> {
    match executor.execute(&args) {
        Ok(output) => {
            if is_error_output(&output) {
                Err(format!("Contract call failed:\n\n{}", output))
            } else {
                let value = parse_return_value(&output);
                Ok(CallOutput { value, raw_output: output })
            }
        }
        Err(e) => Err(format!("Contract call failed: {}", e)),
    }
}
```

### Server Integration

In the MCP server handler, convert to `CallToolResult`:

```rust
use rmcp::handler::server::wrapper::Json;

// For structured output
let result = deploy_contract(&executor, params)?;
Json(result)  // Automatically becomes CallToolResult::structured()

// For simple success/fail
let result = build_contract(&executor, params);
match result {
    Ok(()) => "Build successful!",  // Becomes CallToolResult::success()
    Err(e) => return Err(e),        // Becomes CallToolResult::error()
}
```

### Tests Become Type-Safe

```rust
#[test]
fn deploy_returns_address() -> Result<()> {
    let result = deploy_contract(&executor, params)?;
    assert!(result.address.starts_with("0x") || result.address.starts_with("5"));
    Ok(())
}

#[test]
fn call_returns_state() -> Result<()> {
    let result = call_contract(&executor, params)?;
    assert_eq!(result.value, "false");
    Ok(())
}

#[test]
fn build_succeeds() -> Result<()> {
    build_contract(&executor, params)?;  // Just check it doesn't error
    Ok(())
}
```

### Option 2: Custom ToolOutcome Type

If we need more control, implement a custom type:

```rust
pub enum ToolOutcome<T> {
    Success { data: T, message: String },
    Error { message: String },
}

impl<T: Serialize + JsonSchema> IntoCallToolResult for ToolOutcome<T> {
    fn into_call_tool_result(self) -> Result<CallToolResult, ErrorData> {
        match self {
            ToolOutcome::Success { data, message } => {
                let value = serde_json::to_value(data)?;
                // Include message in content for LLM, data in structured_content for parsing
                Ok(CallToolResult {
                    content: vec![Content::text(message)],
                    structured_content: Some(value),
                    is_error: Some(false),
                    meta: None,
                })
            }
            ToolOutcome::Error { message } => {
                Ok(CallToolResult::error(vec![Content::text(message)]))
            }
        }
    }
}
```

## Tool Categories

| Category | Tools | Output Type | Test Assertion |
|----------|-------|-------------|----------------|
| Binary | `build`, `clean`, `test` | `Result<(), String>` | `result.is_ok()` |
| Address | `deploy`, `convert` | `Result<AddressOutput, String>` | `result?.address` |
| Value | `call` | `Result<CallOutput, String>` | `result?.value` |
| URL | `up` (chain) | `Result<NodeOutput, String>` | `result?.url` |

## Benefits

1. **Type-safe tests** - No string parsing in test assertions
2. **Schema validation** - MCP clients can validate structured output
3. **Backward compatible** - `content` field still contains human-readable text
4. **LLM friendly** - Error messages remain in `content` for LLM consumption
5. **Testable** - Tool functions return typed data, easy to unit test

## Migration Path

1. Define output types for each tool category
2. Update tool functions to return `Result<Output, String>`
3. Update server handlers to use `Json<T>` wrapper
4. Update tests to use typed assertions
5. Add `outputSchema` to tool definitions

## References

- [MCP Tools Specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools.md)
- [rmcp crate](https://crates.io/crates/rmcp) v0.8.4
- [rmcp source](~/.cargo/registry/src/*/rmcp-0.8.4/)
