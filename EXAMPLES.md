# Pop MCP Server - Usage Examples

This document provides practical examples of how to use the Pop MCP Server through Claude or other AI assistants.

**Source Code**: [https://github.com/r0gue-io/pop-mcp](https://github.com/r0gue-io/pop-mcp)

## Installation & Setup Examples

### Check if Pop CLI is Installed

**User Prompt:**
```
Is Pop CLI installed on my system?
```

**What happens:**
- Claude uses the `check_pop_installation` tool
- Returns version info if installed, or installation instructions if not

---

### Get Installation Instructions

**User Prompt:**
```
I need to install Pop CLI on macOS. Can you help?
```

**What happens:**
- Claude uses the `install_pop_instructions` tool with platform="macos"
- Provides step-by-step Homebrew installation instructions

---

## Project Creation Examples

### List Available Templates

**User Prompt:**
```
What contract templates are available for ink!?
```

**What happens:**
- Claude uses the `list_templates` tool
- Shows all 7 templates with descriptions

---

### Create an ERC20 Token Contract

**User Prompt:**
```
Create a new ERC20 token contract called MyToken
```

**What happens:**
- Claude uses `create_contract` with:
  - name: "MyToken"
  - template: "erc20"
  - path: current directory
- Creates a new directory with the ERC20 contract scaffold

---

### Create an NFT Contract in a Specific Location

**User Prompt:**
```
Create an ERC721 NFT contract called CoolNFT in /Users/dev/projects/
```

**What happens:**
- Claude uses `create_contract` with:
  - name: "CoolNFT"
  - template: "erc721"
  - path: "/Users/dev/projects/"

---

## Development Workflow Examples

### Build a Contract

**User Prompt:**
```
Build the contract in ./my-contract
```

**What happens:**
- Claude uses `build_contract` with path="./my-contract"
- Compiles the contract and shows build output

---

### Build for Production

**User Prompt:**
```
Build my contract with optimizations for production
```

**What happens:**
- Claude uses `build_contract` with:
  - path: detected from context or asked
  - release: true
- Creates optimized WASM build

---

### Run Unit Tests

**User Prompt:**
```
Run the tests for my contract at ./token-contract
```

**What happens:**
- Claude uses `test_contract` with:
  - path: "./token-contract"
  - e2e: false
- Runs standard Rust unit tests

---

### Run End-to-End Tests

**User Prompt:**
```
Run e2e tests for the contract
```

**What happens:**
- Claude uses `test_contract` with:
  - path: detected or asked
  - e2e: true
- Launches a local node and runs e2e tests

---

### Get Contract Information

**User Prompt:**
```
What's the metadata for my built contract?
```

**What happens:**
- Claude uses `get_contract_info`
- Reads and displays contract metadata (name, version, authors)

---

## Deployment Examples

### Deploy to Local Node

**User Prompt:**
```
Deploy my contract to a local node using Alice's account
```

**What happens:**
- Claude uses `deploy_contract` with:
  - path: detected
  - constructor: "new"
  - suri: "//Alice"
  - url: "ws://localhost:9944"
- Launches local node if needed and deploys

---

### Deploy with Constructor Arguments

**User Prompt:**
```
Deploy the ERC20 contract with initial supply of 1000000
```

**What happens:**
- Claude uses `deploy_contract` with:
  - path: detected
  - constructor: "new"
  - args: "1000000"
- Deploys with the specified initial supply

---

### Dry Run Deployment

**User Prompt:**
```
Can you estimate the gas cost for deploying this contract?
```

**What happens:**
- Claude uses `deploy_contract` with:
  - path: detected
  - dryRun: true
- Shows gas estimation without actually deploying

---

### Deploy to a Public Network

**User Prompt:**
```
Deploy my contract to Contracts on Polkadot at wss://polkadot-contracts-rpc.polkadot.io
```

**What happens:**
- Claude uses `deploy_contract` with:
  - path: detected
  - url: "wss://polkadot-contracts-rpc.polkadot.io"
  - suri: will ask for your account key
- Deploys to the public network

---

## Contract Interaction Examples

### Call a Contract Method

**User Prompt:**
```
Call the 'transfer' method on contract 5GrwvaEF5zXb... with args: recipient=5FHn... amount=100
```

**What happens:**
- Claude uses `call_contract` with:
  - contract: "5GrwvaEF5zXb..."
  - message: "transfer"
  - args: "5FHn... 100"
- Executes the transfer

---

### Query Contract State

**User Prompt:**
```
What's the balance of account 5FHn... in the ERC20 contract?
```

**What happens:**
- Claude uses `call_contract` with:
  - contract: detected
  - message: "balance_of"
  - args: "5FHn..."
  - dryRun: true (read-only query)
- Returns the balance without modifying state

---

## Documentation Query Examples

### Ask About ink! Features

**User Prompt:**
```
How do I define storage in an ink! contract?
```

**What happens:**
- Claude reads from `ink://docs/llm-guide` resource
- Provides detailed explanation with code examples
- May also reference technical guide for advanced details

---

### Ask About XCM Integration

**User Prompt:**
```
How can I send XCM messages from my ink! contract?
```

**What happens:**
- Claude reads from `xcm://docs/comprehensive-guide` and `xcm://docs/ink-examples`
- Explains XCM concepts
- Shows real contract code examples

---

### Ask About Testing Strategies

**User Prompt:**
```
What's the best way to test ink! contracts?
```

**What happens:**
- Claude reads from ink! documentation resources
- Explains unit tests, integration tests, and e2e tests
- Provides testing patterns and best practices

---

## Complete Workflow Examples

### Full Project Setup to Deployment

**User Prompt:**
```
I want to create a new ERC20 token, build it, test it, and deploy it locally. The token should be called HackToken with 1 million initial supply.
```

**What Claude Does:**

1. **Check Prerequisites**
   - Uses `check_pop_installation` to verify Pop CLI

2. **Create Project**
   - Uses `create_contract` with name="HackToken" template="erc20"

3. **Build Contract**
   - Uses `build_contract` with release=true

4. **Run Tests**
   - Uses `test_contract` to verify contract works

5. **Deploy Locally**
   - Uses `deploy_contract` with args="1000000"

6. **Provide Summary**
   - Shows contract address
   - Explains next steps
   - Suggests how to interact with the contract

---

### Debug a Failed Build

**User Prompt:**
```
My contract build is failing. Can you help?
```

**What Claude Does:**

1. **Attempt Build**
   - Uses `build_contract` to see the error

2. **Analyze Error**
   - Reads ink! documentation for common issues
   - Identifies the problem (e.g., syntax error, missing dependency)

3. **Suggest Fixes**
   - Provides specific code corrections
   - May reference documentation sections

4. **Verify Fix**
   - Runs `build_contract` again after you apply fixes

---

### Create a Multi-Contract System

**User Prompt:**
```
I want to create a voting system with two contracts: one for managing proposals and one for token-weighted voting
```

**What Claude Does:**

1. **Plan Architecture**
   - Explains contract separation
   - Describes cross-contract communication

2. **Create Contracts**
   - Uses `create_contract` for first contract (proposal manager)
   - Uses `create_contract` for second contract (voting contract)

3. **Provide Implementation Guidance**
   - Reads from cross-contract-calls documentation
   - Shows how to set up contract references
   - Explains message patterns

4. **Build Both**
   - Uses `build_contract` for each

5. **Deployment Strategy**
   - Explains deployment order
   - Shows how to link contracts after deployment

---

## Advanced Examples

### Benchmarking and Optimization

**User Prompt:**
```
My contract is using too much gas. How can I optimize it?
```

**What Claude Does:**
- Reads ink! optimization documentation
- Suggests code patterns for gas efficiency
- May recommend testing with `dryRun: true` to compare gas costs
- Uses `build_contract` with release=true for final optimization

---

### Setting Up E2E Testing

**User Prompt:**
```
Set up end-to-end testing for my contract
```

**What Claude Does:**
- Explains e2e test structure from documentation
- Shows example test code
- Uses `test_contract` with e2e=true to run tests
- Helps debug any test failures

---

### Multi-Network Deployment

**User Prompt:**
```
Deploy my contract to Aleph Zero testnet
```

**What Claude Does:**
- Looks up Aleph Zero testnet RPC endpoint
- Uses `deploy_contract` with appropriate URL
- Provides network-specific guidance
- Saves deployment info for future reference

---

## Tips for Best Results

1. **Be Specific**: Include paths, contract names, and specific parameters
2. **Context Matters**: Mention if you're in a contract directory or need absolute paths
3. **Ask for Explanations**: Claude can read documentation to explain concepts
4. **Iterative Development**: Fix issues step by step with Claude's help
5. **Combine Operations**: Ask for complete workflows and Claude will use multiple tools

## Getting Help

If you encounter issues:

```
Can you help me troubleshoot this error: [paste error message]
```

Claude will:
- Analyze the error
- Search documentation for solutions
- Suggest specific fixes
- Help verify the solution works
