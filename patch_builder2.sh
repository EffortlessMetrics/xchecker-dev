cat << 'INNER_EOF' > patch2.diff
--- crates/xchecker-packet/src/builder.rs
+++ crates/xchecker-packet/src/builder.rs
@@ -512,12 +512,14 @@
         line_count: line_count_raw,
         byte_count: byte_count_raw,
     };
+
+    let mut insights_generated = false;

     // Cache Logic Inlined
     let file_content = if let Some(cache_mutex) = cache {
         // Try to get cached insights
         let cached_insights = {
             let mut guard = cache_mutex.lock().expect("Cache mutex poisoned");
             // Pass None for logger to avoid Sync issues in threads
             guard.get_insights(&selected_file.path, &blake3_pre_redaction, phase, None)?
         };
@@ -536,6 +538,7 @@
         } else {
             // Cache miss
+            insights_generated = true;
             let redaction_result = redactor.redact_content(&content, candidate.path.as_ref())?;
             let redacted_content = redaction_result.content;

@@ -588,6 +591,7 @@
     let content_size = file_content.len() + candidate.path.as_str().len() + 10;
     let line_count = file_content.lines().count() + 3;

+    let selected_file_content = if insights_generated { content } else { content.clone() };
+    let mut selected_file = selected_file;
+    selected_file.content = selected_file_content;
+
     Ok(Some((
         selected_file,
INNER_EOF
git restore crates/xchecker-packet/src/builder.rs
patch -p0 < patch2.diff
cargo bench -p xchecker-packet
