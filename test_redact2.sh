#!/bin/bash
cat << 'INNER_EOF' > patch11.diff
--- crates/xchecker-redaction/src/lib.rs
+++ crates/xchecker-redaction/src/lib.rs
@@ -467,7 +467,7 @@

         for index in matches.iter() {
             if let Some((_, regex)) = self.patterns_linear.get(index) {
-                redacted = regex.replace_all(&redacted, "***").into_owned();
+                redacted = std::borrow::Cow::Owned(regex.replace_all(&redacted, "***").into_owned());
             }
         }

INNER_EOF
git restore crates/xchecker-redaction/src/lib.rs
patch -p0 < patch11.diff
cargo bench -p xchecker-redaction
