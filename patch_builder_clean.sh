git checkout crates/xchecker-packet/src/builder.rs
cat << 'INNER_EOF' > patch17.diff
--- crates/xchecker-packet/src/builder.rs
+++ crates/xchecker-packet/src/builder.rs
@@ -504,15 +504,6 @@
     let line_count_raw = content.lines().count();
     let byte_count_raw = content.len();

-    let selected_file = SelectedFile {
-        path: candidate.path.clone(),
-        content: content.clone(), // Clone needed for SelectedFile
-        priority: candidate.priority,
-        blake3_pre_redaction: blake3_pre_redaction.clone(),
-        line_count: line_count_raw,
-        byte_count: byte_count_raw,
-    };
-
     // Cache Logic Inlined
     let file_content = if let Some(cache_mutex) = cache {
         // Try to get cached insights
@@ -519,7 +510,7 @@
         let cached_insights = {
             let mut guard = cache_mutex.lock().expect("Cache mutex poisoned");
             // Pass None for logger to avoid Sync issues in threads
-            guard.get_insights(&selected_file.path, &blake3_pre_redaction, phase, None)?
+            guard.get_insights(&candidate.path, &blake3_pre_redaction, phase, None)?
         };

         if let Some(insights) = cached_insights {
@@ -578,6 +569,15 @@
         let redaction_result = redactor.redact_content(&content, candidate.path.as_ref())?;
         redaction_result.content
     };
+
+    let selected_file = SelectedFile {
+        path: candidate.path.clone(),
+        content,
+        priority: candidate.priority,
+        blake3_pre_redaction,
+        line_count: line_count_raw,
+        byte_count: byte_count_raw,
+    };

     let content_size = file_content.len() + candidate.path.as_str().len() + 10;
     let line_count = file_content.lines().count() + 3;
INNER_EOF
patch -p0 < patch17.diff
cargo bench -p xchecker-packet
