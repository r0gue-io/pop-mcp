# Pop-MCP Agent E2E Test Flows

Use this document to run long, tool-only E2E flows in fresh agent sessions. Each flow is 10–15 tool calls and focuses on chain and contract work. You must use pop-mcp tools only.

Each flow has two parts:

- **Agent Prompt**: human-language instructions you give the subagent.
- **Expected Tool Calls**: the ordered list of tools the agent should end up using. You compare this list with the agent’s reported calls to detect gaps or drift.

## Prerequisites

- Pop CLI installed and available on PATH
- Network access for downloading binaries and runtimes
- Enough local resources to run nodes and zombienet

## Agent Rules (MUST)

- Use pop-mcp tools only. Do not call Pop CLI directly.
- Record every tool call and its output.
- Always clean up nodes and networks at the end of each flow.
- If a step depends on a prior output, paste the exact value into the next step.

## Standard Report Format

Use this format for every flow.

**Checklist**

| Step | Tool | Params (summary) | Result | Key Output |
|------|------|------------------|--------|------------|
| 1 | `check_pop_installation` | none | pass | version: x.y.z |

**Short narrative**

Write 3–5 sentences. Say what worked, what was slow or surprising, and any improvements you would make.

## Flow 1: Local Ink Node + Contract + Chain Queries

Agent prompt:

Start a local ink! node, deploy the existing contract in the repository root, and interact with it multiple times. Then read chain metadata, query `Balances.ExistentialDeposit`, read `System.Account` for Alice, and read `System.Events` to confirm activity. Clean everything at the end.

Expected tool calls (order matters):

1. `check_pop_installation`
2. `up_ink_node`
3. `call_chain` (metadata)
4. `call_chain` (balances constant)
5. `deploy_contract`
6. `call_contract` (read)
7. `call_contract` (write)
8. `call_contract` (read)
9. `call_chain` (storage account)
10. `call_chain` (events)
11. `clean_nodes`

## Flow 2: Paseo + Asset Hub Transfer + Relay Sanity

Agent prompt:

Launch a local paseo network with asset hub. On asset hub, read metadata, query `Balances.ExistentialDeposit`, read `System.Account` for Alice and Bob, do a `Balances.transferKeepAlive` from Alice to Bob, then re-check Bob and read `System.Events`. Also query relay metadata once. Clean the network.

Expected tool calls (order matters):

1. `check_pop_installation`
2. `up_network`
3. `call_chain` (asset hub metadata)
4. `call_chain` (asset hub constant)
5. `call_chain` (asset hub storage Alice)
6. `call_chain` (asset hub storage Bob)
7. `call_chain` (asset hub transfer)
8. `call_chain` (asset hub storage Bob)
9. `call_chain` (asset hub events)
10. `call_chain` (relay metadata)
11. `clean_network`

## Flow 3: Mixed Contract + Asset Hub Cross-Checks

Agent prompt:

Run a local ink! node and a paseo + asset hub network at the same time. Deploy the existing contract on the ink! node, then read state before and after a write call. On asset hub, do a `Balances.transferKeepAlive` from Alice to Bob and confirm `System.Events`. Query relay metadata and relay events. Shut down both systems.

Expected tool calls (order matters):

1. `check_pop_installation`
2. `up_network`
3. `up_ink_node`
4. `deploy_contract`
5. `call_contract` (read)
6. `call_contract` (write)
7. `call_contract` (read)
8. `call_chain` (asset hub metadata)
9. `call_chain` (asset hub transfer)
10. `call_chain` (asset hub events)
11. `call_chain` (relay metadata)
12. `call_chain` (relay events)
13. `clean_nodes`
14. `clean_network`

## Flow 4: Multi-Parachain Network + Deep Queries

Agent prompt:

Launch paseo with two system parachains and query metadata, constants, and account storage on each chain. On both asset hub and coretime, read `Balances.ExistentialDeposit`, `Timestamp.Now`, and `System.Account` for Alice. Confirm relay events, then tear everything down.

Expected tool calls (order matters):

1. `check_pop_installation`
2. `up_network`
3. `call_chain` (relay metadata)
4. `call_chain` (asset hub metadata)
5. `call_chain` (coretime metadata)
6. `call_chain` (asset hub constant)
7. `call_chain` (coretime constant)
8. `call_chain` (asset hub timestamp now)
9. `call_chain` (coretime timestamp now)
10. `call_chain` (asset hub storage account)
11. `call_chain` (coretime storage account)
12. `call_chain` (relay events)
13. `clean_network`

## Flow 5: Full Contract Lifecycle with Multiple Calls

Agent prompt:

Deploy the existing contract, perform multiple state-changing calls, verify the final state, then check `System.Events` for contract execution events. Also query `Balances.ExistentialDeposit` to confirm chain constants are readable. Clean up the node.

Expected tool calls (order matters):

1. `check_pop_installation`
2. `up_ink_node`
3. `deploy_contract`
4. `call_contract` (read)
5. `call_contract` (write)
6. `call_contract` (write)
7. `call_contract` (read)
8. `call_chain` (balances constant)
9. `call_chain` (events)
10. `clean_nodes`

## Flow 6: Utilities + Contract + Chain Checks

Agent prompt:

Convert addresses both ways, then deploy and call the existing contract on a local ink! node. Query metadata, `Balances.ExistentialDeposit`, and `System.Events` before cleaning up.

Expected tool calls (order matters):

1. `check_pop_installation`
2. `convert_address`
3. `convert_address`
4. `up_ink_node`
5. `deploy_contract`
6. `call_contract` (read)
7. `call_chain` (metadata)
8. `call_chain` (balances constant)
9. `call_chain` (events)
10. `clean_nodes`

## Flow 7: Relay + Asset Hub Runtime Health Sweep

Agent prompt:

Launch paseo with asset hub. On the relay and asset hub, verify metadata loads, read `Timestamp.Now`, and read `System.Events`. Then do a small transfer on asset hub and confirm Bob’s balance changed. Clean the network.

Expected tool calls (order matters):

1. `check_pop_installation`
2. `up_network`
3. `call_chain` (relay metadata)
4. `call_chain` (relay timestamp now)
5. `call_chain` (relay events)
6. `call_chain` (asset hub metadata)
7. `call_chain` (asset hub timestamp now)
8. `call_chain` (asset hub events)
9. `call_chain` (asset hub transfer)
10. `call_chain` (asset hub storage Bob)
11. `clean_network`

## Flow 8: Double Contract Deploy + State Verification

Agent prompt:

On a local ink! node, deploy the existing contract twice (separate deployments). For each deployment, read initial state, perform a write, then re-read state to confirm the change. Finally, query `System.Events` and clean up the node.

Expected tool calls (order matters):

1. `check_pop_installation`
2. `up_ink_node`
3. `deploy_contract`
4. `call_contract` (read)
5. `call_contract` (write)
6. `call_contract` (read)
7. `deploy_contract`
8. `call_contract` (read)
9. `call_contract` (write)
10. `call_contract` (read)
11. `call_chain` (events)
12. `clean_nodes`

## Coverage Matrix

| Tool | Flow(s) |
|------|---------|
| `check_pop_installation` | 1–8 |
| `create_contract` | none |
| `build_contract` | none |
| `deploy_contract` | 1, 3, 5, 6, 8 |
| `call_contract` | 1, 3, 5, 6, 8 |
| `up_ink_node` | 1, 3, 5, 6, 8 |
| `call_chain` | 1–8 |
| `clean_nodes` | 1, 3, 5, 6, 8 |
| `up_network` | 2, 3, 4, 7 |
| `clean_network` | 2, 3, 4, 7 |
| `convert_address` | 6 |
| `pop_help` | Optional before any flow |
