cat << 'INNER_EOF' > patch14.diff
--- crates/xchecker-packet/src/builder.rs
+++ crates/xchecker-packet/src/builder.rs
@@ -588,6 +588,14 @@
     let content_size = file_content.len() + candidate.path.as_str().len() + 10;
     let line_count = file_content.lines().count() + 3;

+    let selected_file = SelectedFile {
+        path: candidate.path.clone(),
+        content: file_content.clone(),
+        priority: candidate.priority,
+        blake3_pre_redaction,
+        line_count: line_count_raw,
+        byte_count: byte_count_raw,
+    };
+
     Ok(Some((
         selected_file,
INNER_EOF
git restore crates/xchecker-packet/src/builder.rs
patch -p0 < patch14.diff
cargo bench -p xchecker-packet
