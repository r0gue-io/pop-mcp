# Pop-MCP Tool Addition Plan

Workflow for adding new tools. Implementation patterns in `CLAUDE.md`.

---

## Before Starting

**Create tasks for tracking using TaskCreate.** This ensures no steps are skipped.

**Tasks to create:**

1. **Phase 1: Research**
   - 1.1 Docs Agent (parallel)
   - 1.2 Code Agent (parallel)
   - 1.3 Usage Agent
   - 1.4 User Approval

2. **Phase 2: Implement**
   - 2.1 Create tool file (`src/tools/...`)
   - 2.2 Register in `mod.rs` and `server.rs`
   - 2.3 Add unit tests
   - 2.4 Add integration tests
   - 2.5 Run format, lint, unit tests
   - 2.6 Run integration tests

3. **Phase 3: Code Review**
   - 3.1 Spawn Rust Engineer Agent
   - 3.2 Fix issues until review passes

4. **Phase 4: Verify**
   - 4.1 Run all integration tests
   - 4.2 Fresh Claude session E2E test

**Mark each task complete (TaskUpdate) as you go.**

---

## Phase 1: Research

> **⚠️ Source of Truth:** `pop-cli` codebase is the source of truth. Docs (`pop-docs`) may be outdated. If discrepancies are found between docs and code, note them in the report as a side note for the team to fix.

### Agent Flow

```
Docs Agent ──────┬──────> Code Agent ──────┬──────> Usage Agent ──────> Spec
(what users see) │       (how CLI works)   │       (AI design)
                 └─────────────────────────┘
                      provides context to
```

Run Docs Agent and Code Agent **in parallel**. Usage Agent runs after both complete.

### 1. Docs Agent
**Source:** `../pop-docs`

**Role:** Understand how users interact with this command - examples, common workflows, what problems it solves.

**Output:**
```
## Command: [pop command]
## Purpose: [what problem it solves]
## User Examples: [from docs]
## Common Workflows: [typical usage patterns]
```

**Handoff:** Docs output is passed to Code Agent and Usage Agent.

---

### 2. Code Agent
**Source:** `../pop-cli` (SOURCE OF TRUTH)

**Role:** Research the exact CLI implementation. Identify which flags are **required** for the command to work vs **optional** enhancements. Cross-reference with Docs Agent output and note any discrepancies.

**Input:** Docs Agent output (understands the command's purpose)

**Important:**
- If docs and code disagree, code wins. Add discrepancies to output as `## Docs Discrepancies` section.
- **Interactive prompts:** Pop-MCP tool calls CANNOT respond to interactive prompts from the CLI. Identify any interactive behavior (confirmations, user input prompts, interactive menus) and report in `## Interactive Behavior` section. These must be bypassed via flags (e.g., `-y`, `--skip-confirm`) or the command is unsuitable for pop-mcp.

**Output:**
```
## Command Signature: [exact CLI syntax]
## Required Flags: [must have for command to work]
  - flag: type - why required
## Optional Flags: [enhance but not needed]
  - flag: type - what it does - default value
## Interactive Behavior: [prompts/confirmations that block non-interactive use]
  - what triggers it - how to bypass (flag) or "no bypass available"
## Validation: [what CLI validates]
## Success Output: [what success looks like]
## Error Output: [what failure looks like]
## Docs Discrepancies: [differences between docs and code - for team to fix]
```

**Handoff:** Code output is passed to Usage Agent.

---

### 3. Usage Agent
**Source:** pop-mcp codebase + `CLAUDE.md`

**Role:** Design the tool from an AI user perspective. Understands why pop-mcp exists (AI usability layer). Decides which flags to include and proposes e2e test.

**Input:** Docs Agent output + Code Agent output

**Output:**
```
## Tool Name: [MCP tool name]
## Description: [1-line for AI discovery]
## Flags to Include:
  - [flag]: [required/optional] - [why include]
## Flags to Exclude:
  - [flag]: [why exclude - e.g., "happy path", "AI can't use, simplicity"]
## Params:
  - name: type - description
## Workflow: [how AI uses this with other tools]
## E2E Test: [concrete test scenario]
  Example: "Create contract → build it" tests build_contract
```

---

### Gate: User Approval

After Usage Agent, present **concise spec** for approval:

Spec:
```
## Tool: [name]
## Command: [pop command]
## Include: [flags]
## Exclude: [flags + reason]
## E2E Test: [scenario]
```

User approves → Phase 2

---

## Phase 2: Implement

1. Create tool file (`src/tools/...`) following `CLAUDE.md`
2. Register in `mod.rs` and `server.rs`
3. Add **unit tests** inline with `#[cfg(test)]`
4. Add **integration tests** (`tests/tools/...`)
5. Run: `cargo +nightly fmt && cargo clippy --all-features --all-targets && cargo test`
6. Run the integration tests: `cargo test --features=pop-e2e [tool_name]`

### Unit Tests vs Integration Tests

**Unit tests** (inline `#[cfg(test)]`):
- Test validation logic (empty name, invalid chars, mismatched params)
- Test argument building
- Fast, no CLI execution

**Integration tests** (`tests/tools/`):
- Test **actual CLI behavior** - do NOT duplicate unit test coverage
- Typically only 2 tests needed:
  1. **Success case**: CLI executes and produces expected output/files
  2. **CLI failure case**: CLI returns error (e.g., nonexistent template)
- Do NOT test validation failures here (already covered by unit tests)
- Do NOT use `#[ignore]` - slow tests are expected and must run in CI

**Gate:** All commands pass

---

## Phase 3: Code Review

Spawn a **Rust Engineer Agent** to review the implementation.

**Role:** Expert Rust engineer reviewing for correctness, conciseness, maintainability, and security.

**Checklist:**
- [ ] Follows `CLAUDE.md` patterns (schemars attributes, error handling, arg building)
- [ ] No panics in tool code (returns `error_result` for failures)
- [ ] Input validation covers edge cases
- [ ] No security vulnerabilities (command injection, path traversal)
- [ ] Code is concise - no over-engineering
- [ ] Unit tests cover validation and arg building
- [ ] Integration tests ONLY test CLI behavior (no validation duplication)

**Output:**
```
## Review: [PASS/FAIL]
## Issues Found: [list or "None"]
## Suggestions: [optional improvements]
```

**Gate:** Review passes (so if the review failed, fix the comments, then ask for another review until it passes)

---

## Phase 4: Verify

### 1. Run all integration tests
```bash
cargo test --features=pop-e2e
```

### 2. Fresh agent E2E test
Build and test the tool from a fresh sub-agent session:

```bash
cargo build --release
```

Spawn a sub-agent to run the E2E scenario and report results.

This verifies:
- Tool is discoverable via MCP
- Parameters work as expected
- Output is useful for AI workflows

**Gate:** Both pass (pre-existing test failures noted but not blocking)

---

## Invocation

```
"Add [TOOL] to pop-mcp"
```
