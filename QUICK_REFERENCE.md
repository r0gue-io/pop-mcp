# Pop MCP Server - Quick Reference

**Source Code**: [https://github.com/r0gue-io/pop-mcp](https://github.com/r0gue-io/pop-mcp)

## Installation

```bash
# Clone and build
git clone https://github.com/r0gue-io/pop-mcp.git
cd pop-mcp
npm install
npm run build
```

## Configuration

**Claude Desktop**: Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "pop-cli": {
      "type": "stdio",
      "command": "node",
      "args": [
        "/absolute/path/to/pop-mcp/build/index.js"
      ],
      "env": {}
    }
  }
}
```

**Claude Code (VS Code)**: Add to `~/.claude.json`:

```json
{
  "mcpServers": {
    "pop-cli": {
      "type": "stdio",
      "command": "node",
      "args": [
        "/absolute/path/to/pop-mcp/build/index.js"
      ],
      "env": {}
    }
  }
}
```

## Available Tools

### Installation & Setup
| Tool | Purpose | Key Parameters |
|------|---------|----------------|
| `check_pop_installation` | Verify Pop CLI installed | None |
| `install_pop_instructions` | Get install guide | `platform` (macos/linux/source) |

### Smart Contract Development
| Tool | Purpose | Key Parameters |
|------|---------|----------------|
| `list_templates` | Show contract templates | None |
| `create_contract` | Create new contract | `name`, `template`, `path?` |
| `build_contract` | Build contract | `path`, `release?` |
| `test_contract` | Run tests | `path`, `e2e?`, `node?` |
| `deploy_contract` | Deploy to network | `path`, `constructor?`, `args?`, `suri?`, `url?` |
| `call_contract` | Call contract method | `contract`, `message`, `args?`, `suri?`, `url?` |
| `get_contract_info` | Show contract metadata | `path` |

### Parachain Development
| Tool | Purpose | Key Parameters |
|------|---------|----------------|
| `create_parachain` | Create new parachain | `name`, `template?`, `path?` |
| `build_parachain` | Build parachain | `path`, `paraId?`, `release?` |
| `launch_parachain` | Launch test network | `path?` |

### Pallet Development
| Tool | Purpose | Key Parameters |
|------|---------|----------------|
| `create_pallet` | Create custom pallet | `name`, `path?`, `authors?` |
| `benchmark_pallet` | Run pallet benchmarks | `path`, `pallet?`, `runtime?` |

### Chain Interaction
| Tool | Purpose | Key Parameters |
|------|---------|----------------|
| `call_chain` | Interact with chain | `pallet`, `function`, `args?`, `url?`, `suri?` |

### Utilities
| Tool | Purpose | Key Parameters |
|------|---------|----------------|
| `pop_help` | Get Pop CLI help | `command?` |

## Available Resources

| URI | Description |
|-----|-------------|
| `ink://docs/llm-guide` | Complete ink! language docs |
| `ink://docs/technical-guide` | Technical implementation details |
| `pop://docs/cli-guide` | Pop CLI documentation |
| `xcm://docs/comprehensive-guide` | XCM theory and patterns |
| `xcm://docs/ink-examples` | XCM contract examples |

## Templates

### Contract Templates
- `standard` - Basic flipper contract
- `erc20` - Fungible token (PSP22)
- `erc721` - NFT
- `erc1155` - Multi-token
- `dns` - Domain Name Service
- `cross-contract-calls` - Inter-contract calls
- `multisig` - Multi-signature wallet

### Parachain Templates
- `standard` - Basic parachain with essential pallets
- `assets` - Parachain with asset management (pallet-assets)
- `contracts` - Parachain with smart contract support (pallet-contracts)
- `evm` - EVM-compatible parachain

## Common Workflows

### Smart Contract: Create & Deploy

```
1. Create: "Create an erc20 contract called MyToken"
2. Build: "Build the contract at ./MyToken"
3. Test: "Run tests for the contract"
4. Deploy: "Deploy to local node"
```

### Parachain: Setup & Launch

```
1. Create: "Create a parachain with contracts support called MyChain"
2. Build: "Build the parachain at ./MyChain"
3. Launch: "Launch the parachain locally"
```

### Chain Interaction

```
"Query the balance of account 5GrwvaEF... on Polkadot"
"Call Balances.transfer on my local chain"
"Read the System.Version constant"
```

### Documentation & Learning

```
"How do I implement storage in ink!?"
"Show me XCM examples"
"Explain the difference between contracts and parachains"
```

## Example Prompts

### Contracts
```
"Create a new ERC721 NFT contract"
"Build my contract with optimizations"
"Deploy with initial supply of 1000000"
"Call the transfer method on contract 5Grw..."
```

### Parachains
```
"Create a parachain with asset management"
"Build my parachain for testnet with para ID 2000"
"Create a custom pallet called rewards"
"Benchmark the Balances pallet"
```

### Chain Calls
```
"Query total issuance on Polkadot mainnet"
"Execute a balance transfer on local chain"
"Read runtime version"
```

## File Structure

```
pop-mcp/
├── src/index.ts              # Main server code
├── build/                    # Compiled output
├── .claude/docs/             # Documentation resources
├── package.json              # Dependencies & scripts
└── *.md                      # Documentation
```

## Commands

```bash
npm run build                 # Build the server
npm run watch                 # Watch for changes
npm run inspector            # Test with MCP Inspector
node build/index.js          # Run server directly
```

## Environment

- **Node**: 18+
- **Pop CLI**: 0.11.0+
- **MCP SDK**: ^1.0.4

## Default Values

- **Constructor**: `new`
- **Suri**: `//Alice`
- **URL**: `ws://localhost:9944`
- **Release**: `false`
- **E2E**: `false`
- **Dry Run**: `false`

## Links

- [Pop CLI](https://github.com/r0gue-io/pop-cli)
- [Pop Docs](https://learn.onpop.io)
- [ink! Docs](https://use.ink)
- [MCP Protocol](https://modelcontextprotocol.io)

## Tips

1. Always use absolute paths for contract directories
2. Build before deploying
3. Use `dryRun: true` to estimate gas
4. Test locally before deploying to public networks
5. Check Pop CLI installation first if commands fail

## Support

- Issues: Check TEST.md for troubleshooting
- Examples: See EXAMPLES.md for detailed use cases
- Setup: See SETUP.md for configuration help
