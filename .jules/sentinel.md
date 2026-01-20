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

---

## 2026-01-20 - Symlink Traversal via Non-Existent Paths in SandboxRoot

**Vulnerability:** `SandboxRoot::join()` allowed symlink traversal escape when `allow_symlinks=true` and the target path did not exist. Because non-existent paths bypass `canonicalize()`, a symlinked directory in the path could redirect to outside the sandbox.

**Severity:** HIGH (when `allow_symlinks` is enabled)

**Attack Scenario:**
1. Sandbox root at `/workspace`
2. Attacker creates symlink: `/workspace/escape_dir` → `/tmp/attacker_controlled`
3. Attacker calls `root.join("escape_dir/new_malicious_file.txt")`
4. Old code: path doesn't exist → skip canonicalization → allow the path
5. Result: attacker can write to `/tmp/attacker_controlled/new_malicious_file.txt`

**Root Cause:** The original `SandboxRoot::join()` implementation assumed that for non-existent paths, rejecting `..` components was sufficient. However, symlinks in ancestor directories can redirect the path outside the sandbox without using `..`.

**Learning:** `std::fs::canonicalize()` requires the path to exist. When validating a non-existent path in a sandbox, you must canonicalize and validate the nearest existing ancestor directory, especially when symlinks are allowed.

**Fix Applied:**
1. Added `validate_ancestor_within_sandbox()` method
2. For non-existent paths when `allow_symlinks=true`, find the longest existing prefix
3. Canonicalize that ancestor and verify it stays within the sandbox root
4. Fail with `EscapeAttempt` error if ancestor escapes

**Prevention:** When validating paths that may not exist yet, always validate existing ancestor directories. Don't assume lexical checks (no `..`) are sufficient when symlinks are allowed.

**Files Changed:**
- `crates/xchecker-utils/src/paths.rs` - Added `validate_ancestor_within_sandbox()` and called it for non-existent paths
