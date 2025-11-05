#!/usr/bin/env node

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListResourcesRequestSchema,
  ListToolsRequestSchema,
  ReadResourceRequestSchema,
  ErrorCode,
  McpError,
} from "@modelcontextprotocol/sdk/types.js";
import { execSync } from "child_process";
import { readFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Get the project root (one level up from build directory)
const PROJECT_ROOT = join(__dirname, "..");

// Documentation files paths
const DOCS_PATH = join(PROJECT_ROOT, ".claude", "docs");

const DOCS = [
  {
    uri: "ink://docs/llm-guide",
    name: "ink! Comprehensive Guide",
    description: "Complete ink! smart contract language documentation",
    path: join(DOCS_PATH, "ink-llms.txt"),
    mimeType: "text/plain",
  },
  {
    uri: "ink://docs/technical-guide",
    name: "ink! Technical Guide",
    description: "Deep technical reference for ink! implementation details",
    path: join(DOCS_PATH, "ink-technical-guide.txt"),
    mimeType: "text/plain",
  },
  {
    uri: "pop://docs/cli-guide",
    name: "Pop CLI Comprehensive Guide",
    description: "Complete Pop CLI documentation for Polkadot development",
    path: join(DOCS_PATH, "pop-cli-comprehensive-guide.txt"),
    mimeType: "text/plain",
  },
  {
    uri: "xcm://docs/comprehensive-guide",
    name: "XCM Comprehensive Guide",
    description: "Cross-chain messaging theory, patterns, and best practices",
    path: join(DOCS_PATH, "xcm-comprehensive-guide.txt"),
    mimeType: "text/plain",
  },
  {
    uri: "xcm://docs/ink-examples",
    name: "XCM ink! Examples Guide",
    description: "Real-world XCM contract examples with complete code",
    path: join(DOCS_PATH, "xcm-ink-examples-guide.txt"),
    mimeType: "text/plain",
  },
];

// Helper to execute shell commands
function executeCommand(command: string): { stdout: string; stderr: string; success: boolean } {
  try {
    const stdout = execSync(command, {
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
      maxBuffer: 10 * 1024 * 1024, // 10MB buffer
    });
    return { stdout, stderr: "", success: true };
  } catch (error: any) {
    return {
      stdout: error.stdout?.toString() || "",
      stderr: error.stderr?.toString() || error.message,
      success: false,
    };
  }
}

// Create server instance
const server = new Server(
  {
    name: "pop-mcp-server",
    version: "1.0.0",
  },
  {
    capabilities: {
      resources: {},
      tools: {},
    },
  }
);

// List available resources (documentation files)
server.setRequestHandler(ListResourcesRequestSchema, async () => {
  return {
    resources: DOCS.map((doc) => ({
      uri: doc.uri,
      name: doc.name,
      description: doc.description,
      mimeType: doc.mimeType,
    })),
  };
});

// Read resource content
server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
  const doc = DOCS.find((d) => d.uri === request.params.uri);

  if (!doc) {
    throw new McpError(ErrorCode.InvalidRequest, `Unknown resource: ${request.params.uri}`);
  }

  try {
    const content = readFileSync(doc.path, "utf-8");
    return {
      contents: [
        {
          uri: request.params.uri,
          mimeType: doc.mimeType,
          text: content,
        },
      ],
    };
  } catch (error: any) {
    throw new McpError(
      ErrorCode.InternalError,
      `Failed to read resource: ${error.message}`
    );
  }
});

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "check_pop_installation",
        description: "Check if Pop CLI is installed and get version information",
        inputSchema: {
          type: "object",
          properties: {},
        },
      },
      {
        name: "install_pop_instructions",
        description: "Get detailed instructions for installing Pop CLI on different platforms",
        inputSchema: {
          type: "object",
          properties: {
            platform: {
              type: "string",
              description: "Platform: 'macos', 'linux', or 'source' (for building from source)",
              enum: ["macos", "linux", "source"],
            },
          },
        },
      },
      {
        name: "list_templates",
        description: "List all available ink! contract templates",
        inputSchema: {
          type: "object",
          properties: {},
        },
      },
      {
        name: "create_contract",
        description: "Create a new ink! smart contract from a template using Pop CLI",
        inputSchema: {
          type: "object",
          properties: {
            name: {
              type: "string",
              description: "Name of the contract project",
            },
            template: {
              type: "string",
              description: "Template to use (standard, erc20, erc721, erc1155, dns, cross-contract-calls, multisig)",
              enum: ["standard", "erc20", "erc721", "erc1155", "dns", "cross-contract-calls", "multisig"],
            },
            path: {
              type: "string",
              description: "Directory path where to create the contract (defaults to current directory)",
            },
          },
          required: ["name", "template"],
        },
      },
      {
        name: "build_contract",
        description: "Build an ink! smart contract using Pop CLI",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "Path to the contract directory",
            },
            release: {
              type: "boolean",
              description: "Build in release mode with optimizations (default: false)",
            },
          },
          required: ["path"],
        },
      },
      {
        name: "test_contract",
        description: "Run tests for an ink! smart contract",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "Path to the contract directory",
            },
            e2e: {
              type: "boolean",
              description: "Run end-to-end tests (default: false, runs unit tests)",
            },
            node: {
              type: "string",
              description: "Path to local node for e2e tests (optional)",
            },
          },
          required: ["path"],
        },
      },
      {
        name: "deploy_contract",
        description: "Deploy and instantiate an ink! smart contract to a network",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "Path to the contract directory or .contract bundle",
            },
            constructor: {
              type: "string",
              description: "Constructor function to call (default: 'new')",
            },
            args: {
              type: "string",
              description: "Constructor arguments as space-separated values",
            },
            suri: {
              type: "string",
              description: "Secret key URI for signing (default: //Alice)",
            },
            url: {
              type: "string",
              description: "WebSocket URL of the node (default: ws://localhost:9944)",
            },
            dryRun: {
              type: "boolean",
              description: "Perform a dry run without submitting (default: false)",
            },
            uploadOnly: {
              type: "boolean",
              description: "Only upload code without instantiating (default: false)",
            },
          },
          required: ["path"],
        },
      },
      {
        name: "clean_contract",
        description: "Clean build artifacts for a contract",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "Path to the contract directory",
            },
          },
          required: ["path"],
        },
      },
      {
        name: "get_contract_info",
        description: "Get information about a built contract (metadata, size, etc.)",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "Path to the contract directory or .contract file",
            },
          },
          required: ["path"],
        },
      },
      {
        name: "call_contract",
        description: "Call a contract method on a deployed contract",
        inputSchema: {
          type: "object",
          properties: {
            contract: {
              type: "string",
              description: "Contract address",
            },
            message: {
              type: "string",
              description: "Message/method to call",
            },
            args: {
              type: "string",
              description: "Method arguments as space-separated values",
            },
            suri: {
              type: "string",
              description: "Secret key URI for signing (default: //Alice)",
            },
            url: {
              type: "string",
              description: "WebSocket URL of the node (default: ws://localhost:9944)",
            },
            dryRun: {
              type: "boolean",
              description: "Perform a dry run without submitting (default: false)",
            },
          },
          required: ["contract", "message"],
        },
      },
      {
        name: "create_parachain",
        description: "Create a new parachain/appchain project using Pop CLI templates",
        inputSchema: {
          type: "object",
          properties: {
            name: {
              type: "string",
              description: "Name of the parachain project",
            },
            template: {
              type: "string",
              description: "Template to use: standard, assets, contracts, evm",
              enum: ["standard", "assets", "contracts", "evm"],
            },
            path: {
              type: "string",
              description: "Directory path where to create the parachain (defaults to current directory)",
            },
          },
          required: ["name"],
        },
      },
      {
        name: "build_parachain",
        description: "Build a parachain binary and runtime",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "Path to the parachain directory",
            },
            paraId: {
              type: "number",
              description: "Parachain ID for relay chain onboarding (optional)",
            },
            release: {
              type: "boolean",
              description: "Build in release mode (default: true)",
            },
          },
          required: ["path"],
        },
      },
      {
        name: "launch_parachain",
        description: "Launch a local parachain network for testing",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "Path to the parachain directory (optional)",
            },
          },
        },
      },
      {
        name: "create_pallet",
        description: "Create a new custom pallet for runtime development",
        inputSchema: {
          type: "object",
          properties: {
            name: {
              type: "string",
              description: "Name of the pallet",
            },
            path: {
              type: "string",
              description: "Path where to create the pallet (defaults to current directory)",
            },
            authors: {
              type: "string",
              description: "Pallet authors (optional)",
            },
          },
          required: ["name"],
        },
      },
      {
        name: "call_chain",
        description: "Interact with a Polkadot chain: execute extrinsics, query storage, or read constants",
        inputSchema: {
          type: "object",
          properties: {
            pallet: {
              type: "string",
              description: "Pallet name (e.g., 'System', 'Balances')",
            },
            function: {
              type: "string",
              description: "Function/storage/constant name",
            },
            args: {
              type: "string",
              description: "Arguments as space-separated values or hex strings (optional)",
            },
            url: {
              type: "string",
              description: "WebSocket URL of the chain (default: ws://localhost:9944)",
            },
            suri: {
              type: "string",
              description: "Secret key URI for signing transactions (optional, for extrinsics)",
            },
            sudo: {
              type: "boolean",
              description: "Wrap call in sudo.sudo() (default: false)",
            },
            execute: {
              type: "boolean",
              description: "Execute as transaction (for state-changing calls, default: false)",
            },
          },
          required: ["pallet", "function"],
        },
      },
      {
        name: "benchmark_pallet",
        description: "Run benchmarks for pallets to generate accurate weight information",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "Path to the parachain/chain directory",
            },
            pallet: {
              type: "string",
              description: "Specific pallet to benchmark (optional, prompts if not provided)",
            },
            runtime: {
              type: "string",
              description: "Path to runtime WASM file (optional)",
            },
          },
          required: ["path"],
        },
      },
      {
        name: "pop_help",
        description: "Get help for any Pop CLI command",
        inputSchema: {
          type: "object",
          properties: {
            command: {
              type: "string",
              description: "Command to get help for (e.g., 'new contract', 'new parachain', 'build', 'up', 'call')",
            },
          },
        },
      },
      {
        name: "search_documentation",
        description: "Search through all Polkadot documentation (ink!, Pop CLI, XCM) for specific topics or keywords. Use this to find relevant information across all guides.",
        inputSchema: {
          type: "object",
          properties: {
            query: {
              type: "string",
              description: "Search query or topic (e.g., 'storage', 'cross-contract calls', 'XCM', 'testing')",
            },
            scope: {
              type: "string",
              description: "Optional: Limit search to specific documentation ('ink', 'pop', 'xcm', or 'all' for everything)",
              enum: ["ink", "pop", "xcm", "all"],
            },
          },
          required: ["query"],
        },
      },
    ],
  };
});

// Handle tool calls
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case "check_pop_installation": {
        const result = executeCommand("pop --version");
        if (result.success) {
          return {
            content: [
              {
                type: "text",
                text: `Pop CLI is installed:\n${result.stdout}\n\nRun 'pop --help' for more information.`,
              },
            ],
          };
        } else {
          return {
            content: [
              {
                type: "text",
                text: `Pop CLI is not installed or not in PATH.\n\nError: ${result.stderr}\n\nUse the 'install_pop_instructions' tool to get installation instructions.`,
              },
            ],
          };
        }
      }

      case "install_pop_instructions": {
        const platform = (args as any).platform || "macos";
        let instructions = "";

        if (platform === "macos") {
          instructions = `# Installing Pop CLI on macOS

## Method 1: Using Homebrew (Recommended)

1. Install Homebrew if not already installed:
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

2. Add Homebrew to PATH (Apple Silicon):
   echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
   eval "$(/opt/homebrew/bin/brew shellenv)"

3. Install Pop CLI:
   brew install r0gue-io/pop-cli/pop

4. Verify installation:
   pop --version

5. Set up dependencies:
   pop install

## Method 2: Build from Source
See the 'source' platform option for instructions.`;
        } else if (platform === "linux") {
          instructions = `# Installing Pop CLI on Linux

## Method 1: Using Homebrew

1. Install Homebrew:
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

2. Add Homebrew to PATH:
   echo 'eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"' >> ~/.profile
   eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"

3. Install Pop CLI:
   brew install r0gue-io/pop-cli/pop

4. Verify installation:
   pop --version

5. Set up dependencies:
   pop install

## Method 2: Build from Source
See the 'source' platform option for instructions.`;
        } else if (platform === "source") {
          instructions = `# Building Pop CLI from Source

Works on all platforms (macOS, Linux, Windows with WSL):

1. Install Rust:
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env

2. Build and install Pop CLI:
   cargo install --force --locked pop-cli

3. Verify installation:
   pop --version

4. Set up dependencies:
   pop install

This will compile Pop CLI from source and install it in ~/.cargo/bin/`;
        }

        return {
          content: [
            {
              type: "text",
              text: instructions,
            },
          ],
        };
      }

      case "list_templates": {
        const templates = [
          {
            name: "standard",
            description: "Basic flipper contract - simple boolean toggle",
          },
          {
            name: "erc20",
            description: "ERC-20 fungible token implementation",
          },
          {
            name: "erc721",
            description: "ERC-721 NFT (non-fungible token) implementation",
          },
          {
            name: "erc1155",
            description: "ERC-1155 multi-token standard implementation",
          },
          {
            name: "dns",
            description: "Domain Name Service contract example",
          },
          {
            name: "cross-contract-calls",
            description: "Example of calling between contracts",
          },
          {
            name: "multisig",
            description: "Multi-signature wallet contract",
          },
        ];

        const text = templates
          .map((t) => `• ${t.name}: ${t.description}`)
          .join("\n");

        return {
          content: [
            {
              type: "text",
              text: `Available ink! Contract Templates:\n\n${text}\n\nUse 'create_contract' tool with the template name to create a new contract.`,
            },
          ],
        };
      }

      case "create_contract": {
        const { name, template, path } = args as any;
        const targetPath = path || process.cwd();

        const cmd = `cd "${targetPath}" && pop new contract ${name} --template ${template}`;
        const result = executeCommand(cmd);

        if (result.success) {
          return {
            content: [
              {
                type: "text",
                text: `Successfully created ${template} contract: ${name}\n\nOutput:\n${result.stdout}\n\nNext steps:\n1. cd ${name}\n2. Build: pop build\n3. Test: pop test\n4. Deploy: pop up`,
              },
            ],
          };
        } else {
          throw new McpError(
            ErrorCode.InternalError,
            `Failed to create contract: ${result.stderr}`
          );
        }
      }

      case "build_contract": {
        const { path, release } = args as any;
        const releaseFlag = release ? "--release" : "";

        const cmd = `cd "${path}" && pop build ${releaseFlag}`;
        const result = executeCommand(cmd);

        if (result.success) {
          return {
            content: [
              {
                type: "text",
                text: `Build successful!\n\n${result.stdout}\n\nBuild artifacts are in: ${path}/target/ink/`,
              },
            ],
          };
        } else {
          return {
            content: [
              {
                type: "text",
                text: `Build failed:\n\n${result.stderr}\n\nStdout:\n${result.stdout}`,
              },
            ],
          };
        }
      }

      case "test_contract": {
        const { path, e2e, node } = args as any;
        const e2eFlag = e2e ? "--e2e" : "";
        const nodeFlag = node ? `--node ${node}` : "";

        const cmd = `cd "${path}" && pop test ${e2eFlag} ${nodeFlag}`;
        const result = executeCommand(cmd);

        return {
          content: [
            {
              type: "text",
              text: result.success
                ? `Tests passed!\n\n${result.stdout}`
                : `Tests failed:\n\n${result.stderr}\n\nStdout:\n${result.stdout}`,
            },
          ],
        };
      }

      case "deploy_contract": {
        const { path, constructor, args: ctorArgs, suri, url, dryRun, uploadOnly } = args as any;

        let cmd = `pop up contract --path "${path}"`;
        if (constructor) cmd += ` --constructor ${constructor}`;
        if (ctorArgs) cmd += ` --args ${ctorArgs}`;
        if (suri) cmd += ` --suri "${suri}"`;
        if (url) cmd += ` --url ${url}`;
        if (dryRun) cmd += " --dry-run";
        if (uploadOnly) cmd += " --upload-only";

        const result = executeCommand(cmd);

        return {
          content: [
            {
              type: "text",
              text: result.success
                ? `Deployment successful!\n\n${result.stdout}`
                : `Deployment failed:\n\n${result.stderr}\n\nStdout:\n${result.stdout}`,
            },
          ],
        };
      }

      case "clean_contract": {
        const { path } = args as any;
        const cmd = `cd "${path}" && cargo clean`;
        const result = executeCommand(cmd);

        return {
          content: [
            {
              type: "text",
              text: result.success
                ? `Cleaned build artifacts in ${path}`
                : `Failed to clean: ${result.stderr}`,
            },
          ],
        };
      }

      case "get_contract_info": {
        const { path } = args as any;

        // Try to find contract files
        const findCmd = `find "${path}" -name "*.contract" -o -name "*.json" | grep -E "target/ink|metadata"`;
        const findResult = executeCommand(findCmd);

        let info = `Contract directory: ${path}\n\n`;

        if (findResult.success && findResult.stdout) {
          info += `Found contract files:\n${findResult.stdout}\n\n`;

          // Try to read metadata if available
          const metadataCmd = `find "${path}" -name "*.json" -path "*/target/ink/*" | head -1`;
          const metadataResult = executeCommand(metadataCmd);

          if (metadataResult.success && metadataResult.stdout.trim()) {
            const metadataFile = metadataResult.stdout.trim();
            try {
              const metadata = readFileSync(metadataFile, "utf-8");
              const metadataJson = JSON.parse(metadata);
              info += `Contract Name: ${metadataJson.contract?.name || "Unknown"}\n`;
              info += `Version: ${metadataJson.contract?.version || "Unknown"}\n`;
              if (metadataJson.contract?.authors) {
                info += `Authors: ${metadataJson.contract.authors.join(", ")}\n`;
              }
            } catch (e) {
              info += "Could not parse metadata\n";
            }
          }
        } else {
          info += "No built contract found. Run 'build_contract' first.\n";
        }

        return {
          content: [
            {
              type: "text",
              text: info,
            },
          ],
        };
      }

      case "call_contract": {
        const { contract, message, args: methodArgs, suri, url, dryRun } = args as any;

        let cmd = `pop call contract --contract ${contract} --message ${message}`;
        if (methodArgs) cmd += ` --args ${methodArgs}`;
        if (suri) cmd += ` --suri "${suri}"`;
        if (url) cmd += ` --url ${url}`;
        if (dryRun) cmd += " --dry-run";

        const result = executeCommand(cmd);

        return {
          content: [
            {
              type: "text",
              text: result.success
                ? `Contract call successful!\n\n${result.stdout}`
                : `Contract call failed:\n\n${result.stderr}\n\nStdout:\n${result.stdout}`,
            },
          ],
        };
      }

      case "create_parachain": {
        const { name, template, path } = args as any;
        const targetPath = path || process.cwd();
        const templateFlag = template ? `--template ${template}` : "";

        const cmd = `cd "${targetPath}" && pop new parachain ${name} ${templateFlag}`;
        const result = executeCommand(cmd);

        if (result.success) {
          return {
            content: [
              {
                type: "text",
                text: `Successfully created parachain: ${name}\n\nOutput:\n${result.stdout}\n\nNext steps:\n1. cd ${name}\n2. Build: pop build\n3. Launch: pop up parachain`,
              },
            ],
          };
        } else {
          throw new McpError(
            ErrorCode.InternalError,
            `Failed to create parachain: ${result.stderr}`
          );
        }
      }

      case "build_parachain": {
        const { path, paraId, release } = args as any;
        const paraIdFlag = paraId ? `--para_id ${paraId}` : "";
        const releaseFlag = release === false ? "" : "--release";

        const cmd = `cd "${path}" && pop build ${releaseFlag} ${paraIdFlag}`;
        const result = executeCommand(cmd);

        return {
          content: [
            {
              type: "text",
              text: result.success
                ? `Parachain build successful!\n\n${result.stdout}\n\nBinaries are in: ${path}/target/${release === false ? "debug" : "release"}/`
                : `Build failed:\n\n${result.stderr}\n\nStdout:\n${result.stdout}`,
            },
          ],
        };
      }

      case "launch_parachain": {
        const { path } = args as any;
        const pathArg = path ? `--path "${path}"` : "";

        return {
          content: [
            {
              type: "text",
              text: `To launch a local parachain network, run:\n\npop up parachain ${pathArg}\n\nThis will start a local relay chain and parachain for testing.\n\nNote: This is a long-running process that will block the terminal. Press Ctrl+C to stop.\n\nThe parachain will be accessible at:\n- Parachain RPC: ws://localhost:9944\n- Relay Chain RPC: ws://localhost:9900`,
            },
          ],
        };
      }

      case "create_pallet": {
        const { name, path, authors } = args as any;
        const targetPath = path || process.cwd();
        const authorsFlag = authors ? `--authors "${authors}"` : "";

        const cmd = `cd "${targetPath}" && pop new pallet ${name} ${authorsFlag}`;
        const result = executeCommand(cmd);

        if (result.success) {
          return {
            content: [
              {
                type: "text",
                text: `Successfully created pallet: ${name}\n\nOutput:\n${result.stdout}\n\nThe pallet has been created and can be integrated into your runtime.`,
              },
            ],
          };
        } else {
          throw new McpError(
            ErrorCode.InternalError,
            `Failed to create pallet: ${result.stderr}`
          );
        }
      }

      case "call_chain": {
        const { pallet, function: func, args: callArgs, url, suri, sudo, execute } = args as any;

        let cmd = `pop call chain --pallet ${pallet} --function ${func}`;
        if (callArgs) cmd += ` --args "${callArgs}"`;
        if (url) cmd += ` --url ${url}`;
        if (suri) cmd += ` --suri "${suri}"`;
        if (sudo) cmd += " --sudo";
        if (!execute) cmd += " -y"; // Skip confirmation for queries

        const result = executeCommand(cmd);

        return {
          content: [
            {
              type: "text",
              text: result.success
                ? `Chain call successful!\n\n${result.stdout}`
                : `Chain call failed:\n\n${result.stderr}\n\nStdout:\n${result.stdout}`,
            },
          ],
        };
      }

      case "benchmark_pallet": {
        const { path, pallet, runtime } = args as any;

        let cmd = `cd "${path}" && pop bench pallet`;
        if (pallet) cmd += ` --pallet=${pallet}`;
        if (runtime) cmd += ` --runtime=${runtime}`;

        const result = executeCommand(cmd);

        return {
          content: [
            {
              type: "text",
              text: result.success
                ? `Benchmark complete!\n\n${result.stdout}`
                : `Benchmark failed:\n\n${result.stderr}\n\nStdout:\n${result.stdout}`,
            },
          ],
        };
      }

      case "pop_help": {
        const { command } = args as any;
        const cmd = command ? `pop ${command} --help` : "pop --help";
        const result = executeCommand(cmd);

        return {
          content: [
            {
              type: "text",
              text: result.success
                ? result.stdout
                : `Failed to get help: ${result.stderr}`,
            },
          ],
        };
      }

      case "search_documentation": {
        const { query, scope } = args as any;
        const searchScope = scope || "all";

        // Filter docs based on scope
        let docsToSearch = DOCS;
        if (searchScope !== "all") {
          docsToSearch = DOCS.filter(doc => {
            if (searchScope === "ink") return doc.uri.startsWith("ink://");
            if (searchScope === "pop") return doc.uri.startsWith("pop://");
            if (searchScope === "xcm") return doc.uri.startsWith("xcm://");
            return true;
          });
        }

        let results = "";
        const queryLower = query.toLowerCase();

        for (const doc of docsToSearch) {
          try {
            const content = readFileSync(doc.path, "utf-8");
            const lines = content.split("\n");
            const matchingLines: string[] = [];

            // Search for matches with context
            for (let i = 0; i < lines.length; i++) {
              if (lines[i].toLowerCase().includes(queryLower)) {
                // Add context: 2 lines before and after
                const start = Math.max(0, i - 2);
                const end = Math.min(lines.length, i + 3);
                const context = lines.slice(start, end).join("\n");
                matchingLines.push(`Line ${i + 1}:\n${context}\n---`);

                // Limit matches per document
                if (matchingLines.length >= 5) break;
              }
            }

            if (matchingLines.length > 0) {
              results += `\n## ${doc.name} (${doc.uri})\n\n`;
              results += matchingLines.join("\n\n");
              results += "\n\n";
            }
          } catch (error) {
            // Skip files that can't be read
            continue;
          }
        }

        if (results) {
          return {
            content: [
              {
                type: "text",
                text: `Search results for "${query}" in ${searchScope} documentation:\n${results}\n\nTo read full documentation, use the resource URIs shown above.`,
              },
            ],
          };
        } else {
          return {
            content: [
              {
                type: "text",
                text: `No results found for "${query}" in ${searchScope} documentation.\n\nAvailable documentation:\n${DOCS.map(d => `- ${d.name}: ${d.uri}`).join("\n")}\n\nTry different keywords or read the full documentation using the resource URIs.`,
              },
            ],
          };
        }
      }

      default:
        throw new McpError(
          ErrorCode.MethodNotFound,
          `Unknown tool: ${name}`
        );
    }
  } catch (error: any) {
    if (error instanceof McpError) {
      throw error;
    }
    throw new McpError(
      ErrorCode.InternalError,
      `Tool execution failed: ${error.message}`
    );
  }
});

// Start server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("Pop MCP Server running on stdio");
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
