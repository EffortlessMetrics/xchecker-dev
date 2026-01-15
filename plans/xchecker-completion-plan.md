# xchecker-dev Route-to-Completion Plan

## Executive Summary

**Project**: xchecker-dev  
**Version**: 1.0.0  
**Purpose**: CLI tool for orchestrating spec generation workflows with Claude AI  
**Current State**: Core implementation complete; V11-V14 LLM backend support implemented; V15-V18 ecosystem features implemented but not fully wired

---

## Project Overview

xchecker is a Rust-based CLI tool that orchestrates spec generation workflows using AI language models. It implements a 6-phase pipeline (Requirements → Design → Tasks → Review → Fixup → Final) with controlled execution, atomic file operations, secret redaction, and cryptographic audit trails.

### Key Architectural Components

1. **LLM Backend Layer** - Trait-based abstraction supporting multiple providers (claude-cli, gemini-cli, openrouter, anthropic)
2. **Phase System** - Trait-based phase implementations with dependency enforcement
3. **Orchestrator** - Manages phase execution, transitions, and state
4. **Runner System** - Cross-platform process execution (Native, WSL, Auto modes)
5. **Packet Builder** - Deterministic context assembly with budget enforcement
6. **Fixup Engine** - Safe file modification with fuzzy matching and atomic writes
7. **Hooks System** - Pre/post-phase hook execution (implemented but not wired)
8. **Workspace System** - Multi-spec orchestration with registry
9. **Template System** - Built-in templates for quick spec bootstrapping
10. **Gate System** - CI/CD policy enforcement

---

## Current Implementation Status

### ✅ Fully Implemented

| Feature | Status | Evidence |
|---------|--------|----------|
| **Core Runtime (V1-V10)** | | |
| - JCS Canonicalization | ✅ Complete | `src/canonicalization.rs` |
| - Secret Redaction | ✅ Complete | `src/redaction.rs` |
| - Runner System | ✅ Complete | `src/runner.rs` with timeout, NDJSON, buffers |
| - Orchestrator | ✅ Complete | `src/orchestrator/mod.rs`, `phase_exec.rs` |
| - Packet Builder | ✅ Complete | `src/packet.rs` |
| - Fixup Engine | ✅ Complete | `src/fixup.rs` with fuzzy matching |
| - Lock Manager | ✅ Complete | `src/lock.rs` with drift detection |
| - Artifact Manager | ✅ Complete | `src/artifact.rs` |
| - Receipt Manager | ✅ Complete | `src/receipt.rs` with JCS emission |
| - Status Manager | ✅ Complete | `src/status.rs` |
| - Config System | ✅ Complete | `src/config.rs` with discovery |
| - Phase System | ✅ Complete | `src/phase.rs`, `phases.rs` |
| - Exit Codes | ✅ Complete | `src/exit_codes.rs` |
| **LLM Backend (V11-V14)** | | |
| - LlmBackend Trait | ✅ Complete | `src/llm/mod.rs` |
| - Claude CLI Backend | ✅ Complete | `src/llm/claude_cli.rs` |
| - Gemini CLI Backend | ✅ Complete | `src/llm/gemini_cli.rs` |
| - OpenRouter Backend | ✅ Complete | `src/llm/openrouter_backend.rs` |
| - Anthropic Backend | ✅ Complete | `src/llm/anthropic_backend.rs` |
| - HTTP Client | ✅ Complete | `src/llm/http_client.rs` |
| - Budgeted Backend | ✅ Complete | `src/llm/budgeted_backend.rs` |
| - LLM Factory | ✅ Complete | `src/llm/mod.rs` `from_config()` |
| **Ecosystem (V16-V18)** | | |
| - Workspace | ✅ Complete | `src/workspace.rs` |
| - Templates | ✅ Complete | `src/template.rs` |
| - Gate Command | ✅ Complete | `src/gate.rs` |
| - Hooks System | ✅ Complete | `src/hooks.rs` (full implementation) |
| **Documentation** | | |
| - LLM Providers Guide | ✅ Complete | `docs/LLM_PROVIDERS.md` |
| - Configuration Guide | ✅ Complete | `docs/CONFIGURATION.md` |
| - Schema Definitions | ✅ Complete | `schemas/receipt.v1.json`, `status.v1.json`, `doctor.v1.json` |

### ⚠️ Partially Implemented / Not Wired

| Feature | Status | Evidence | Notes |
|---------|--------|----------|
| **Hooks Integration** | ⚠️ Implemented but not wired | `src/hooks.rs` fully implemented, but `src/orchestrator/phase_exec.rs` has no hook calls |
| **ExecutionStrategy** | ⚠️ Controlled only | ExternalTool reserved for V15+ |
| **Fallback Provider** | ⚠️ Config parsing supports it | `src/llm/mod.rs` has `fallback_provider: None` comment |
| **Prompt Templates** | ⚠️ Config parsing supports it | `src/llm/mod.rs` has `prompt_template: None` comment |
| **Gemini CLI Flags** | ⚠️ Config parsing incomplete | `src/cli.rs` has `llm_gemini_binary: None` TODO |
| **TODO Items Found** | | |
| - Windows hardlink detection | `src/fixup.rs:1536` - "TODO: Implement proper Windows hardlink detection" |
| - Claude CLI version extraction | `src/orchestrator/llm.rs:219` - "TODO: Extract from extensions if available" |
| - Runner distro extraction | `src/orchestrator/llm.rs:221` - "TODO: Extract from extensions if available" |
| - Config test isolation issues | `src/config.rs:1864, 1962, 2008, 2036` - Tests marked with environment isolation issues |

---

## Critical Blockers and Dependencies

### Blockers

1. **Hooks Not Wired in Orchestrator**
   - Impact: Users cannot use pre/post-phase hooks despite full implementation
   - Resolution: Wire `HookExecutor` into `execute_phase_core()` in `phase_exec.rs`
   - Complexity: Medium - Requires understanding orchestrator execution flow

2. **ExecutionStrategy Limited to Controlled**
   - Impact: ExternalTool/agentic mode reserved for V15+
   - Resolution: This is intentional for V11-V14; no action needed

3. **Fallback Provider Not Active**
   - Impact: Config supports `fallback_provider` but factory doesn't use it
   - Resolution: Implement fallback logic in `LlmBackend::from_config()` or document as unsupported

4. **Prompt Templates Not Active**
   - Impact: Config supports `prompt_template` but not used
   - Resolution: Implement template selection in backend factory or document as unsupported

5. **Gemini CLI Binary Flag Missing**
   - Impact: No CLI flag for `--llm-gemini-binary`
   - Resolution: Add flag to `src/cli.rs` and wire through config

6. **Environment Isolation in Config Tests**
   - Impact: Some tests have environment isolation issues
   - Resolution: Fix tests or document as expected behavior

### Dependencies

- **No external dependencies** - All work is internal to the codebase
- **Test framework** - Uses built-in Rust testing tools

---

## Prioritized Completion Plan

### Phase 1: Immediate Cleanup (High Priority)

1. **Wire Hooks into Orchestrator**
   - Add hook execution calls in `execute_phase_core()` before and after phase execution
   - Record hook warnings in receipts
   - Test pre/post-phase hook execution

2. **Remove TODO Comments and Dead Code**
   - Remove or implement Windows hardlink detection in `src/fixup.rs:1536`
   - Remove or implement Claude CLI version extraction in `src/orchestrator/llm.rs:219`
   - Remove or implement runner distro extraction in `src/orchestrator/llm.rs:221`
   - Fix or document environment isolation issues in config tests

3. **Add Gemini CLI Binary Flag**
   - Add `--llm-gemini-binary` flag to `src/cli.rs`
   - Wire through config system to `LlmConfig.gemini.binary`

### Phase 2: Documentation Updates (Medium Priority)

1. **Update LLM Providers Documentation**
   - Document that all 4 providers are fully supported (not reserved)
   - Add examples for each provider configuration
   - Document OpenRouter budget enforcement (XCHECKER_OPENROUTER_BUDGET)

2. **Create Hooks Integration Guide**
   - Document how to configure hooks in `.xchecker/config.toml`
   - Provide examples of pre/post-phase hooks
   - Document hook failure behavior (warn vs fail)

3. **Update README with Current State**
   - Clarify V11-V14 features are complete
   - Document hooks as implemented but not yet wired
   - Document that ExternalTool is reserved for V15+

### Phase 3: Enhanced LLM Backend (Low Priority)

1. **Implement Fallback Provider Support**
   - Add fallback logic to `LlmBackend::from_config()` in `src/llm/mod.rs`
   - Test fallback from primary to secondary provider
   - Record fallback usage in receipts (add `fallback_used` warning)

2. **Implement Prompt Template Support**
   - Add prompt template parsing to config system
   - Implement template validation (compatibility rules)
   - Pass template to backend factory for prompt formatting

3. **Implement Per-Phase Model Selection**
   - Add support for `model_requirements`, `model_design`, etc. in config
   - Wire per-phase model selection in `LlmInvocation` metadata

### Phase 4: Test Coverage Enhancements (Low Priority)

1. **Add LLM Integration Tests**
   - Create tests for each LLM backend (claude-cli, gemini-cli, openrouter, anthropic)
   - Add tests for fallback provider logic
   - Add tests for prompt template system

2. **Add Hooks Integration Tests**
   - Create tests for pre/post-phase hook execution
   - Test hook failure behavior (warn vs fail)
   - Test hook timeout handling

3. **Fix Config Test Isolation**
   - Fix environment isolation issues in config tests
   - Ensure tests use isolated environments

### Phase 5: Documentation Polish (Low Priority)

1. **Create Architecture Documentation**
   - Document LLM backend architecture with Mermaid diagrams
   - Document phase execution flow
   - Document packet building process
   - Document fixup application flow

2. **Create Migration Guide**
   - Document upgrade path from v1.0 to v1.1
   - Document any breaking changes
   - Document config migration steps

---

## Risk Assessment

| Risk | Severity | Mitigation |
|-------|----------|----------|
| Hooks wiring changes orchestrator core | Medium | Comprehensive testing required |
| Windows hardlink detection | Low | Document as known limitation |
| Fallback provider logic | Medium | Requires careful testing of error paths |
| Prompt templates | Low | Breaking changes if not careful |

---

## System Architecture Diagram

```mermaid
graph TD
    A[CLI Entry] --> B[Phase Orchestrator]
    B --> C[LLM Backend Layer]
    B --> D[Phase System]
    B --> E[Runner System]
    B --> F[Fixup Engine]
    B --> G[Hooks System]
    B --> H[Workspace System]
    B --> I[Template System]
    B --> J[Gate System]
    
    C --> C1[Claude CLI Backend]
    C --> C2[Gemini CLI Backend]
    C --> C3[OpenRouter Backend]
    C --> C4[Anthropic Backend]
    
    D --> D1[Requirements Phase]
    D --> D2[Design Phase]
    D --> D3[Tasks Phase]
    D --> D4[Review Phase]
    D --> D5[Fixup Phase]
    
    G -. H1[Hooks Executor]
    G -. H2[Config System]
    
    style C1 fill:#ff6b6b
    style C2 fill:#ff6b6b
    style C3 fill:#ff6b6b
    style C4 fill:#ff6b6b
    
    style G stroke:#ff9999,stroke-width:2px,stroke-dasharray:5 5
    
    class H1 implemented
    class G wired
```

---

## Next Steps

1. Review and approve this plan
2. Switch to Code mode to implement Phase 1 tasks
3. Complete Phase 2-5 tasks as time permits
4. Monitor CI/CD for integration testing

---

## Notes

- The project is in excellent shape with comprehensive core implementation
- Main gaps are integration-related (hooks wiring, fallback, templates)
- All V11-V14 LLM backend features appear to be implemented
- Documentation needs updates to reflect current state
- No critical architectural issues identified
- Test coverage appears comprehensive based on test file structure
