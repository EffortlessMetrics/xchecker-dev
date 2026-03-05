cat << 'INNER_EOF' > patch_fix.diff
--- crates/xchecker-packet/src/builder.rs
+++ crates/xchecker-packet/src/builder.rs
@@ -575,7 +575,7 @@

     let selected_file = SelectedFile {
         path: candidate.path.clone(),
-        content: String::new(), // Populate dynamically or avoid cloning content since it's just used for testing later in build_packet, wait no it is needed later!
+        content,
         priority: candidate.priority,
         blake3_pre_redaction,
         line_count: line_count_raw,
INNER_EOF
patch -p0 < patch_fix.diff
cargo bench -p xchecker-packet
