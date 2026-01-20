## 2026-01-19 - Symlink Traversal in Packet Construction

**Vulnerability:** `ContentSelector::walk_directory` followed symlinks during recursive directory walking, allowing arbitrary file read (path traversal) if a malicious symlink was present in the source tree. This could expose sensitive system files to the LLM context packet.

**Severity:** HIGH

**Root Cause:** The original implementation used `path.is_dir()` which follows symlinks (stat behavior), and `fs::read_to_string()` which also follows symlinks. This meant:
1. A symlink to `/etc/passwd` would be read and included in the packet
2. A symlink to a directory outside the workspace would be recursively traversed

**Learning:** `fs::read_dir` returns entries where `entry.file_type()` reflects the link itself (lstat semantics), but `path.is_dir()` and `path.is_file()` follow links (stat semantics). Recursive walkers must explicitly check `file_type.is_symlink()` to avoid unintended traversal. `fs::read_to_string` also follows links.

**Fix Applied:**
1. Check `file_type.is_symlink()` before processing any entry
2. Default behavior: skip all symlinks (secure-by-default)
3. Optional `allow_symlinks(true)`: validate symlink targets stay within the base directory using `fs::canonicalize()`
4. Fail-closed: broken symlinks or canonicalization errors result in skipping the entry

**Prevention:** Always check `is_symlink()` during directory traversal. Use `SandboxRoot` or equivalent canonicalization checks to ensure symlink targets stay within trust boundaries. Default to rejecting symlinks.

**Files Changed:**
- `crates/xchecker-engine/src/packet/selectors.rs` - Core fix in `walk_directory()`
- `crates/xchecker-engine/src/packet/builder.rs` - API surface for configuration

## 2026-01-20 - Unbounded Memory Consumption in File Selection

**Vulnerability:** `ContentSelector::walk_directory` read entire file contents into memory using `fs::read_to_string` before any size checking. This allowed a malicious or misconfigured repository with large files (e.g., 10GB logs) to cause an Out-Of-Memory (OOM) crash or Denial of Service (DoS) by exhausting system resources.

**Severity:** MEDIUM

**Root Cause:** File reading was eager (read-all-then-check) rather than lazy or bounded. The `ContentSelector` did not enforce any per-file size limits during traversal.

**Learning:** Always check file metadata (`fs::metadata(path).len()`) before reading content into memory. Set hard limits on per-file sizes independent of total budget constraints. Also, avoid reading special files (pipes, devices) which might block indefinitely.

**Fix Applied:**
1. Added `max_file_size` limit to `ContentSelector` (default: 10MB for DoS protection).
2. This limit is **independent** of `packet_max_bytes` - they serve different purposes:
   - `max_file_size`: Prevents reading any single huge file into memory (DoS protection)
   - `packet_max_bytes`: Total budget for all files in the packet (budget constraint)
3. In `walk_directory`, check `metadata.len()` and `metadata.is_file()` before reading.
4. Fail hard if critical `Upstream` files exceed the size limit, but skip non-critical files with a warning.

**Prevention:** Enforce resource limits early in the processing pipeline (fail-fast). Use metadata checks before I/O. Keep per-file DoS limits separate from aggregate budget limits.

**Files Changed:**
- `crates/xchecker-engine/src/packet/selectors.rs`
- `crates/xchecker-engine/src/packet/builder.rs`
