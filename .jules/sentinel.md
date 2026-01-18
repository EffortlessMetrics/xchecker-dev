## 2026-01-18 - Symlink Traversal via Non-Existent Paths
**Vulnerability:** Directory traversal allowed when `allow_symlinks` is true, by accessing a non-existent file through a symlink pointing outside the sandbox.
**Learning:** `std::fs::canonicalize` requires the path to exist. Validating `full_path.exists()` is insufficient because non-existent paths bypass the canonicalization check, assuming they are safe if they don't contain `..`. However, symlinks in the path components can redirect the path outside the sandbox.
**Prevention:** When validating a non-existent path in a sandbox, verify that the nearest existing ancestor directory resolves to a location within the sandbox, especially when symlinks are allowed.
