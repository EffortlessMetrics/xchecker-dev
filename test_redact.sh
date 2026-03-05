#!/bin/bash
cat << 'INNER_EOF' > patch_redact.diff
--- crates/xchecker-redaction/src/lib.rs
+++ crates/xchecker-redaction/src/lib.rs
@@ -677,14 +677,37 @@

-                    // Replace the line in the content
-                    let line_start = content
-                        .lines()
-                        .take(secret_match.line_number - 1)
-                        .map(|l| l.len() + 1) // +1 for newline
-                        .sum::<usize>();
-                    let line_end = line_start + line.len();
-
-                    redacted_content.replace_range(line_start..line_end, &redacted_line);
+                }
+            }
+        }
+        // Wait, the memory says rebuilding strings iterating line by line is faster!
INNER_EOF
