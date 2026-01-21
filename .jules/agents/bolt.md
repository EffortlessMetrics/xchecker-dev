# Bolt's Journal

## 2024-05-22 - [Dependency Policy Violation]
**Learning:** Adding new dependencies (even dev-dependencies or transient ones like `rayon`) requires explicit approval.
**Action:** Always check `Cargo.toml` constraints and ask before adding deps. Use `std::thread::scope` for simple parallelism instead.
