# xchecker Product Overview

xchecker is a Rust CLI tool for orchestrating spec generation workflows using Claude AI. It transforms rough feature ideas into structured requirements, designs, and implementation tasks through an automated multi-phase pipeline.

## What xchecker Does

xchecker is a **spec pipeline with receipts and gateable JSON contracts**. It:
- Takes a rough idea and produces structured requirements, designs, and tasks
- Generates cryptographically-hashed receipts for every LLM interaction
- Emits machine-readable JSON for CI gates and integrations
- Applies code changes through a controlled fixup pipeline (LLMs propose, xchecker applies)

## When to Use xchecker

| Good Fit | Not a Good Fit |
|----------|----------------|
| Feature planning with audit trails | Quick one-off code generation |
| Structured multi-phase workflows | Free-form chat with an LLM |
| CI-integrated spec validation | Projects without Claude CLI access |
| Teams needing reproducible specs | Real-time interactive editing |
| Projects requiring secret scanning | Single-file micro-changes |

## Non-Goals

xchecker explicitly does **not**:
- **Generate code directly**: LLMs propose diffs; xchecker applies them through the fixup engine
- **Execute arbitrary LLM commands**: Only the controlled execution strategy is supported
- **Fail silently**: All failures produce structured errors with exit codes
- **Skip security checks**: Secret scanning runs before every LLM invocation
- **Support "best effort" modes**: Operations either succeed completely or fail with actionable diagnostics

## Core Workflow

The pipeline executes specs through sequential phases:
```
Requirements → Design → Tasks → Review → Fixup → Final
```

Each phase:
1. Builds a packet (context from artifacts and files)
2. Scans for secrets (blocks if detected)
3. Invokes LLM (Claude CLI or dry-run)
4. Postprocesses response into artifacts
5. Writes artifacts atomically via `.partial/` staging
6. Generates receipts with BLAKE3 hashes

## Key Features

- Multi-phase pipeline with dependency tracking
- Versioned JSON contracts (JCS/RFC 8785 canonical emission)
- Automatic secret detection and redaction
- Cross-platform support (Linux, macOS, Windows with WSL)
- Lockfile system for version pinning and drift detection
- Atomic file writes with staging directories

## State Directory

xchecker stores state in `.xchecker/` (configurable via `XCHECKER_HOME`):
```
.xchecker/
  config.toml           # Project configuration
  specs/<spec-id>/
    artifacts/          # Generated phase outputs
    receipts/           # Execution audit trails
    context/            # Packet previews for debugging
```

## Exit Codes

| Code | Name | Description |
|------|------|-------------|
| 0 | SUCCESS | Completed successfully |
| 7 | PACKET_OVERFLOW | Packet size exceeded |
| 8 | SECRET_DETECTED | Secret found in content |
| 9 | LOCK_HELD | Lock already held |
| 10 | PHASE_TIMEOUT | Phase timed out |
| 70 | CLAUDE_FAILURE | Claude CLI failed |
