## 2026-01-19 - Symlink Traversal in Packet Construction
**Vulnerability:** `ContentSelector` followed symlinks during recursive directory walking, allowing arbitrary file read (path traversal) if a malicious symlink was present in the source tree. This could expose sensitive system files to the LLM.
**Learning:** `fs::read_dir` returns entries where `file_type()` reflects the link itself (lstat), but `path.is_dir()` (stat) follows links. Recursive walkers must explicitly check for symlinks to avoid unintended traversal. `fs::read_to_string` also follows links.
**Prevention:** Always check `is_symlink()` during directory traversal. Use `SandboxRoot` or equivalent canonicalization checks to ensure symlink targets stay within trust boundaries. Default to rejecting symlinks.
