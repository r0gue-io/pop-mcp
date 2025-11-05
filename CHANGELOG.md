# Changelog

## v1.0.0 - Complete Polkadot Development Support

### Major Changes

#### Expanded from Smart Contracts to Full Polkadot Development
- **Breaking**: Repositioned from "ink! smart contract tool" to "complete Polkadot development platform"
- Now supports smart contracts, parachains, pallets, and chain interaction

#### New Tools Added (7 new tools, 19 total)

**Parachain Development:**
- `create_parachain` - Create new parachain/appchain projects from 4 templates
- `build_parachain` - Build parachain binaries and runtime
- `launch_parachain` - Launch local parachain networks for testing

**Pallet Development:**
- `create_pallet` - Create custom runtime pallets
- `benchmark_pallet` - Run pallet benchmarks for weight calculation

**Chain Interaction:**
- `call_chain` - Execute extrinsics, query storage, read constants on any Polkadot chain

**Renamed:**
- `pop_up_parachain` → `launch_parachain` (for consistency)

#### Parachain Templates Added
- `standard` - Basic parachain with essential pallets
- `assets` - Asset management with pallet-assets
- `contracts` - Smart contract support with pallet-contracts
- `evm` - EVM-compatible parachain

#### Documentation Updates
- All documentation now emphasizes **complete Polkadot development**
- Added parachain and pallet workflows
- Added chain interaction examples
- Updated all example prompts to cover full capabilities

#### Configuration Updates
- Added **Claude Code (VS Code)** configuration instructions
- Removed local machine-specific paths from examples
- Added NPM installation instructions
- Made all setup guides platform-agnostic

### What's Included

#### Smart Contract Tools (7)
- Contract templates (7): standard, erc20, erc721, erc1155, dns, cross-contract-calls, multisig
- Full contract lifecycle: create, build, test, deploy, call, inspect

#### Parachain Tools (3)
- Parachain templates (4): standard, assets, contracts, evm
- Complete parachain workflow: create, build, launch

#### Pallet Tools (2)
- Create custom pallets
- Run benchmarks

#### Chain Interaction Tools (1)
- Universal chain interaction (any Polkadot chain)

#### Setup & Utilities (6)
- Installation management
- Template listing
- Help system

### Installation

#### NPM (Recommended)
```bash
npm install -g @pop-cli/mcp-server
```

#### From Source
```bash
git clone <repo>
cd pop-mcp
npm install
npm run build
```

### Configuration

#### Claude Desktop
```json
{
  "mcpServers": {
    "pop-cli": {
      "type": "stdio",
      "command": "pop-mcp-server",
      "args": [],
      "env": {}
    }
  }
}
```

#### Claude Code (VS Code)
Add to `~/.claude.json`:
```json
{
  "mcpServers": {
    "pop-cli": {
      "type": "stdio",
      "command": "pop-mcp-server",
      "args": [],
      "env": {}
    }
  }
}
```

### Breaking Changes

None - all existing tools remain compatible. New capabilities are additive.

### Migration Guide

If you were using v0.x focused only on contracts:
- All contract tools work exactly the same
- New capabilities are available through natural language
- No configuration changes needed
- Documentation updated to show full capabilities

### Technical Changes

- Server code expanded from 650 to 1000+ lines
- Added 7 new tool schemas and implementations
- Enhanced error handling for parachain operations
- Improved command construction for complex operations

### Documentation Changes

**Updated Files:**
- README.md - Now emphasizes full Polkadot development
- GETTING_STARTED.md - Added NPM install, removed local paths, added VS Code
- QUICK_REFERENCE.md - Added all new tools and templates
- PROJECT_SUMMARY.md - Updated with new capabilities
- All example prompts updated

**What Users Can Now Do:**
```
// Smart Contracts (existing)
"Create an ERC20 token contract"
"Build and deploy my contract"

// Parachains (NEW!)
"Create a parachain with smart contract support"
"Build my parachain for testnet"
"Launch a local parachain network"

// Pallets (NEW!)
"Create a custom pallet called rewards"
"Benchmark the Balances pallet"

// Chain Interaction (NEW!)
"Query balance on Polkadot mainnet"
"Execute a transfer on my local chain"
"Read the runtime version"
```

### Future Enhancements

Potential v2.0 features:
- Contract upgrade helpers
- Multi-chain deployment orchestration
- Automated testing frameworks
- CI/CD integration tools
- Gas optimization suggestions
- Security analysis tools

### Contributors

Built for the Polkadot hackathon community 🚀

### License

MIT

---

## Pre-v1.0 (Initial Release)

### v0.1.0 - Initial MCP Server for ink! Contracts

- 12 tools for smart contract development
- 5 documentation resources
- 7 contract templates
- Support for Claude Desktop
- Complete contract lifecycle support
