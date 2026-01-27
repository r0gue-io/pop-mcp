# Pop-MCP E2E Test Flows

Run: `./scripts/e2e-flows.sh [1|2|3|4|all]`

## Flow 1: Contract Lifecycle
Create → Build → Test → Deploy → Call

## Flow 2: Chain Lifecycle
Create → Build → Test

## Flow 3: Chain Runtime Interaction
Start node → Query metadata → Read constant → Query storage → Execute transaction → Clean

## Flow 4: Utilities
Check installation → Convert addresses

## Coverage

| Tool | Flow |
|------|------|
| `check_pop_installation` | 4 |
| `create_contract` | 1 |
| `build_contract` | 1 |
| `test_contract` | 1 |
| `deploy_contract` | 1 |
| `call_contract` | 1 |
| `create_chain` | 2 |
| `build_chain` | 2 |
| `test_chain` | 2 |
| `up_ink_node` | 1, 3 |
| `call_chain` | 3 |
| `clean_nodes` | 3 |
| `convert_address` | 4 |
