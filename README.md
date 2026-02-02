# Pop MCP Server

A comprehensive Model Context Protocol (MCP) server for **complete Polkadot development**, providing seamless integration with Pop CLI for smart contracts, parachains, pallets, and chain interaction.

**Source Code**: [https://github.com/r0gue-io/pop-mcp](https://github.com/r0gue-io/pop-mcp)

## Overview

Build anything on Polkadot through natural conversation with your AI assistant:

- 🔷 **Smart Contracts** - Create, build, test, and deploy ink! contracts
- ⛓️ **Parachains** - Build and launch custom parachains/appchains
- 🧩 **Pallets** - Create custom runtime modules
- 🔗 **Chain Interaction** - Call extrinsics, query storage, read constants
- 📚 **Documentation** - Complete Polkadot, ink!, XCM, and Substrate guides
- 🛠️ **Development Tools** - Full lifecycle from idea to deployment

## Features

### 📚 Documentation Resources

Access comprehensive guides directly through your AI assistant:

- **ink! Comprehensive Guide** - Complete smart contract language documentation
- **ink! Technical Guide** - Deep technical implementation details
- **Pop CLI Guide** - Complete tooling for contracts, parachains, and pallets
- **XCM Comprehensive Guide** - Cross-chain messaging theory and patterns
- **XCM ink! Examples** - Real-world XCM contract examples with code

### 🛠️ Development Tools

Powerful tools for complete Polkadot development:

#### Installation & Setup
- `check_pop_installation` - Verify Pop CLI installation
- `install_pop_instructions` - Platform-specific installation guides

#### Smart Contract Development
- `list_templates` - View contract templates (ERC20, ERC721, ERC1155, DNS, multisig, etc.)
- `create_contract` - Create contracts from templates
- `build_contract` - Build contracts with optimization
- `test_contract` - Run unit and e2e tests
- `deploy_contract` - Deploy to any network
- `call_contract` - Execute contract methods
- `get_contract_info` - Inspect contract metadata

#### Parachain Development
- `create_parachain` - Create new parachain/appchain projects
- `build_parachain` - Build parachain binaries and runtime
- `launch_parachain` - Launch local parachain networks

#### Pallet Development
- `create_pallet` - Create custom runtime pallets
- `benchmark_pallet` - Run pallet benchmarks

#### Chain Interaction
- `call_chain` - Execute extrinsics, query storage, read constants
- `query_chain_storage` - Read chain state
- `read_chain_constant` - Read runtime constants

#### Utilities
- `clean_project` - Clean build artifacts
- `pop_help` - Get help for any Pop CLI command
- `search_documentation` - Search through all documentation for topics and keywords

## Installation

### Install the MCP Server

Download the latest pre-built binary for your platform from the [GitHub Releases](https://github.com/r0gue-io/pop-mcp/releases):

```bash
# macOS ARM64 (Apple Silicon)
curl -L https://github.com/r0gue-io/pop-mcp/releases/latest/download/pop-mcp-server-aarch64-apple-darwin.tar.gz | tar xz
chmod +x pop-mcp-server-aarch64-apple-darwin

# macOS Intel
curl -L https://github.com/r0gue-io/pop-mcp/releases/latest/download/pop-mcp-server-x86_64-apple-darwin.tar.gz | tar xz
chmod +x pop-mcp-server-x86_64-apple-darwin

# Linux
curl -L https://github.com/r0gue-io/pop-mcp/releases/latest/download/pop-mcp-server-x86_64-unknown-linux-gnu.tar.gz | tar xz
chmod +x pop-mcp-server-x86_64-unknown-linux-gnu
```

Move the binary to a location in your PATH (optional):

```bash
sudo mv pop-mcp-server-* /usr/local/bin/pop-mcp-server
```

### Build from Source

To build from source:

```bash
# Clone the repository
git clone https://github.com/r0gue-io/pop-mcp.git
cd pop-mcp

# Build with Rust
cargo build --release

# The binary will be at target/release/pop-mcp-server
```

## Configuration

### Claude Desktop Configuration

Add to your Claude Desktop configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "pop-mcp": {
      "type": "stdio",
      "command": "/path/to/pop-mcp-server",
      "args": [],
      "env": {}
    }
  }
}
```

Replace `/path/to/pop-mcp-server` with the actual path to your binary, or just `pop-mcp-server` if it's in your PATH.

### Claude Code Configuration

Claude Code is available in VS Code! To use the Pop MCP Server:

#### Global Configuration (Recommended)

Add the MCP server to your global Claude settings file at `~/.claude.json`:

```json
{
  "mcpServers": {
    "pop-mcp": {
      "type": "stdio",
      "command": "/path/to/pop-mcp-server",
      "args": [],
      "env": {}
    }
  }
}
```

#### Option 2: Project-Specific Configuration

Alternatively, add a `.mcp.json` file in your project directory:

```json
{
  "mcpServers": {
    "pop-mcp": {
      "type": "stdio",
      "command": "/path/to/pop-mcp-server",
      "args": [],
      "env": {}
    }
  }
}
```

#### After Configuration

1. **Restart Claude Code** (or reload VS Code window)
2. Verify the server is loaded with `/mcp` command in Claude Code
3. You should see `pop-mcp` listed with available tools

#### Auto-Loading Documentation Resources (Optional)

To have Claude automatically load ink!, Pop CLI, and XCM documentation at the start of each conversation, create a `CLAUDE.md` file in your project root:

```markdown
# Polkadot Development Context

Automatically load comprehensive documentation for ink! smart contracts and Polkadot development:

@pop-mcp:ink://docs/llm-guide
@pop-mcp:pop://docs/cli-guide

For XCM cross-chain development:
@pop-mcp:xcm://docs/comprehensive-guide
@pop-mcp:xcm://docs/ink-examples
```

This ensures Claude has full access to ink!, Pop CLI, and XCM documentation without needing to manually @-mention resources in each conversation.

Now you can develop Polkadot projects directly in your IDE with Claude's help!

### Cursor Configuration

Cursor is an AI-powered IDE that supports MCP servers. To configure Pop MCP:

#### 1. Add the MCP Server

1. Open **Settings** (⌘/Ctrl + ,)
2. Navigate to **Tools & MCP**
3. Click **Add new MCP Server**
4. Paste the following configuration:

```json
{
  "pop-mcp": {
    "type": "stdio",
    "command": "/path/to/pop-mcp-server",
    "args": [],
    "env": {}
  }
}
```

Replace `/path/to/pop-mcp-server` with the actual path to your binary, or just `pop-mcp-server` if it's in your PATH.

#### 2. Auto-Load Documentation (Recommended)

To have Cursor automatically load Polkadot documentation when working on ink! smart contracts:

1. Open **Settings** (⌘/Ctrl + ,)
2. Navigate to **Rules, Memories, Commands**
3. Click **Add a Project Rule**
4. Set the application mode to **Apply Intelligently**
5. Name it: **When developing ink! smart contracts**
6. Add the following content:

```markdown
Automatically load comprehensive documentation for ink! smart contracts and Polkadot development:

@pop-mcp:ink://docs/llm-guide
@pop-mcp:pop://docs/cli-guide

For XCM cross-chain development:
@pop-mcp:xcm://docs/comprehensive-guide
@pop-mcp:xcm://docs/ink-examples
```

This ensures Cursor's AI has full access to ink!, Pop CLI, and XCM documentation when working on Polkadot projects.

**Tip:** When starting a new chat in Cursor, click the rule name in the composer to manually add it to your conversation. This ensures the Polkadot documentation is loaded into the chat context.

### Other MCP Clients

For other MCP clients (Cline, Zed, etc.), refer to their documentation for adding MCP servers. Generally, you'll need to provide:

- Command: `pop-mcp-server` (or full path: `/path/to/pop-mcp-server`)
- Args: `[]` (empty)

## Usage Examples

### Checking Pop CLI Installation

```
User: Is Pop CLI installed?
Assistant: [Uses check_pop_installation tool]
```

### Creating a New Contract

```
User: Create an ERC20 token contract called MyToken
Assistant: [Uses create_contract with name="MyToken" and template="erc20"]
```

### Building and Testing

```
User: Build and test the contract in ./my-contract
Assistant: [Uses build_contract and test_contract tools]
```

### Deploying a Contract

```
User: Deploy the contract to my local node
Assistant: [Uses deploy_contract with appropriate parameters]
```

### Accessing Documentation

Documentation is automatically available - just ask questions:

```
User: How do I implement storage in ink!?
Assistant: [Claude reads from ink! documentation and explains storage implementation]
```

```
User: What are the best practices for XCM?
Assistant: [Claude accesses XCM comprehensive guide and provides best practices]
```

### Searching Documentation

Use the search tool for specific queries:

```
User: Search for information about testing contracts
Assistant: [Uses search_documentation with query="testing" to find relevant sections]
```

```
User: Find examples of cross-contract calls in the ink! docs
Assistant: [Searches ink! documentation and returns relevant examples with context]
```

## Available Templates

- **standard** - Basic flipper contract (boolean toggle)
- **erc20** - Fungible token implementation
- **erc721** - NFT implementation
- **erc1155** - Multi-token standard
- **dns** - Domain Name Service example
- **cross-contract-calls** - Inter-contract communication
- **multisig** - Multi-signature wallet

## Tool Reference

### Installation Tools

#### check_pop_installation
Checks if Pop CLI is installed and returns version information.

```typescript
// No parameters required
```

#### install_pop_instructions
Provides platform-specific installation instructions.

```typescript
{
  platform?: "macos" | "linux" | "source"  // Optional, defaults to macos
}
```

### Project Creation Tools

#### list_templates
Lists all available ink! contract templates.

```typescript
// No parameters required
```

#### create_contract
Creates a new contract from a template.

```typescript
{
  name: string,           // Contract project name
  template: string,       // Template name (see Available Templates)
  path?: string          // Target directory (defaults to current)
}
```

### Development Tools

#### build_contract
Builds a contract with optional optimizations.

```typescript
{
  path: string,          // Contract directory path
  release?: boolean      // Build with optimizations (default: false)
}
```

#### test_contract
Runs contract tests (unit or e2e).

```typescript
{
  path: string,          // Contract directory path
  e2e?: boolean,        // Run e2e tests (default: false)
  node?: string         // Custom node path for e2e (optional)
}
```

#### clean_contract
Removes build artifacts.

```typescript
{
  path: string          // Contract directory path
}
```

#### get_contract_info
Retrieves contract metadata and information.

```typescript
{
  path: string          // Contract directory or .contract file path
}
```

### Deployment Tools

#### deploy_contract
Deploys and instantiates a contract.

```typescript
{
  path: string,              // Contract directory or .contract bundle
  constructor?: string,      // Constructor name (default: "new")
  args?: string,            // Constructor arguments (space-separated)
  url?: string,             // Node WebSocket URL (default: ws://localhost:9944)
  dryRun?: boolean,         // Dry run without submitting (default: false)
  uploadOnly?: boolean      // Only upload code, don't instantiate (default: false)
}
```

Signing:
- Set `PRIVATE_KEY` in the environment to a dev key URI (e.g. `//Alice`) when `execute=true`.

#### call_contract
Calls a method on a deployed contract.

```typescript
{
  contract: string,         // Contract address
  message: string,          // Method name to call
  args?: string,           // Method arguments (space-separated)
  url?: string,            // Node WebSocket URL (default: ws://localhost:9944)
  dryRun?: boolean         // Dry run without submitting (default: false)
}
```

Signing:
- Set `PRIVATE_KEY` in the environment to a dev key URI (e.g. `//Alice`) when `execute=true`.

### Network Tools

#### pop_up_parachain
Provides instructions for launching a local parachain network.

```typescript
{
  path?: string            // Parachain directory (optional)
}
```

### Utility Tools

#### pop_help
Gets help for any Pop CLI command.

```typescript
{
  command?: string         // Command to get help for (e.g., "new contract")
}
```

#### search_documentation
Searches through all Polkadot documentation for specific topics or keywords.

```typescript
{
  query: string,           // Search query (e.g., "storage", "testing", "XCM")
  scope?: string          // Optional: "ink", "pop", "xcm", or "all" (default: "all")
}
```

## Documentation Resources

The Pop MCP Server provides comprehensive Polkadot documentation that Claude can access in **three ways**:

### 1. Automatic Context (Recommended)
Claude automatically has access to all documentation when the MCP server is loaded. Just ask questions naturally:

```
How do I define storage in an ink! contract?
Explain XCM and how to use it in contracts
What's the difference between contracts and parachains?
Show me how to implement cross-contract calls
```

### 2. Search Tool
Use the `search_documentation` tool to find specific information:

```
Search the documentation for "storage macros"
Find information about XCM in the docs
Look up testing patterns in ink!
```

### 3. Direct Resource URIs
Access specific documentation files via MCP resource protocol:

- `ink://docs/llm-guide` - ink! comprehensive guide (complete language reference)
- `ink://docs/technical-guide` - ink! technical reference (deep implementation details)
- `pop://docs/cli-guide` - Pop CLI documentation (tooling and workflows)
- `xcm://docs/comprehensive-guide` - XCM theory and patterns (cross-chain messaging)
- `xcm://docs/ink-examples` - XCM contract examples (real-world code)

### Available Documentation Topics

Your AI assistant has comprehensive knowledge about:
- **ink! Smart Contracts**: Language syntax, macros, storage, events, errors
- **Contract Patterns**: ERC20, ERC721, ERC1155, DNS, multisig, cross-contract calls
- **Testing**: Unit tests, e2e tests, test patterns and best practices
- **XCM**: Cross-chain messaging, integration patterns, real examples
- **Pop CLI**: All commands, workflows, and tooling
- **Deployment**: Local nodes, testnets, mainnet deployment strategies
- **Optimization**: Gas optimization, storage efficiency, best practices

## Project Structure

```
pop-mcp/
├── src/
│   └── index.ts          # Main MCP server implementation
├── build/                # Compiled JavaScript (generated)
├── .claude/
│   └── docs/            # Documentation resources
│       ├── ink-llms.txt
│       ├── ink-technical-guide.txt
│       ├── pop-cli-comprehensive-guide.txt
│       ├── xcm-comprehensive-guide.txt
│       └── xcm-ink-examples-guide.txt
├── package.json
├── tsconfig.json
└── README.md
```

## Development

### Building

```bash
npm run build
```

### Watching for Changes

```bash
npm run watch
```

### Testing with MCP Inspector

The MCP Inspector provides a UI for testing your server:

```bash
npm run inspector
```

This will open a web interface where you can:
- Test individual tools
- View available resources
- Inspect request/response payloads
- Debug server behavior

## Troubleshooting

### Server Not Connecting

1. Verify the build completed successfully: `npm run build`
2. Check the path in your MCP client configuration
3. Look for errors in the client's developer console or logs
4. Restart your MCP client after configuration changes

### Pop CLI Commands Failing

1. Verify Pop CLI is installed: `pop --version`
2. Ensure you're in the correct directory for contract operations
3. Check that paths provided to tools are absolute paths
4. Review Pop CLI output for specific error messages

### Permission Issues

On macOS/Linux, ensure the build output is executable:

```bash
chmod +x build/index.js
```

## Contributing

This MCP server is designed for hackathons and rapid development. Contributions are welcome!

### Adding New Tools

1. Add tool definition in `ListToolsRequestSchema` handler
2. Implement tool logic in `CallToolRequestSchema` handler
3. Update README documentation
4. Test with MCP Inspector

### Adding Documentation Resources

1. Place documentation files in `.claude/docs/`
2. Add resource definition to `DOCS` array
3. Rebuild the server

## Resources

- [Pop MCP Server GitHub](https://github.com/r0gue-io/pop-mcp) - This repository
- [Pop CLI Documentation](https://learn.onpop.io)
- [Pop CLI GitHub](https://github.com/r0gue-io/pop-cli)
- [ink! Documentation](https://use.ink)
- [Model Context Protocol](https://modelcontextprotocol.io)
- [Polkadot Documentation](https://docs.polkadot.com)

## License

MIT

## Support

For issues and questions:
- Pop MCP Server: https://github.com/r0gue-io/pop-mcp/issues
- Pop CLI: https://github.com/r0gue-io/pop-cli/issues
- MCP Protocol: https://github.com/modelcontextprotocol/specification

---

Built for Polkadot hackathons and developers 🚀
