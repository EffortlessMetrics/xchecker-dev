## 2024-05-20 - [TOCTOU Vulnerability]
**Vulnerability:** [Time-Of-Check to Time-Of-Use]
**Learning:** [When reading files, open the file first and read from the handle rather than checking metadata and then opening it.]
**Prevention:** [Open the file first with `fs::File::open`, check `file.metadata()`, and then read.]
