cat << 'INNER_EOF' > patch_redact.diff
--- crates/xchecker-redaction/src/lib.rs
+++ crates/xchecker-redaction/src/lib.rs
@@ -459,21 +459,20 @@
     /// # Returns
     /// The redacted text with secrets replaced by "***"
     #[must_use]
     pub fn redact_string(&self, text: &str) -> String {
         let matches = self.regex_set.matches(text);
         if !matches.matched_any() {
             return text.to_string();
         }

-        let mut redacted = text.to_string();
+        let mut redacted = std::borrow::Cow::Borrowed(text);

         for index in matches.iter() {
             if let Some((_, regex)) = self.patterns_linear.get(index) {
-                redacted = regex.replace_all(&redacted, "***").to_string();
+                redacted = std::borrow::Cow::Owned(regex.replace_all(&redacted, "***").into_owned());
             }
         }

-        redacted
+        redacted.into_owned()
     }

INNER_EOF
git restore crates/xchecker-redaction/src/lib.rs
patch -p0 < patch_redact.diff
cargo bench -p xchecker-redaction
