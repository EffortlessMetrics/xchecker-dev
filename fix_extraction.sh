sed -i 's/use regex::Regex;/use regex::Regex;\nuse std::sync::LazyLock;/g' crates/xchecker-extraction/src/lib.rs

cat << 'INNEREOF' > patch.txt
<<<<<<< SEARCH
    // Match user story patterns: **User Story:** or **User Story**:
    let user_story_re = Regex::new(r"(?im)^\s*\*\*User\s+Story[:\*]").unwrap();
    summary.user_story_count = user_story_re.find_iter(markdown).count();

    // Match EARS-style acceptance criteria: WHEN ... THEN ... SHALL
    // Also match simpler patterns: GIVEN/WHEN/THEN or numbered criteria with SHALL
    let ears_re =
        Regex::new(r"(?im)(WHEN\s+.+\s+THEN\s+.+\s+SHALL|GIVEN\s+.+\s+WHEN\s+.+\s+THEN)").unwrap();
    summary.acceptance_criteria_count = ears_re.find_iter(markdown).count();

    // Match NFR patterns: **NFR-*, NFR-*, **Non-Functional*
    let nfr_re = Regex::new(r"(?im)(^\s*\*\*NFR[-\s]|\bNFR-\w+\b|^\s*\*\*Non-Functional)").unwrap();
    summary.nfr_count = nfr_re.find_iter(markdown).count();

    // Match requirement headings: ### Requirement N or ## Requirement N (allow leading whitespace)
    let req_heading_re = Regex::new(r"(?im)^\s*#{2,3}\s+Requirement\s+\d+").unwrap();
    summary.requirement_count = req_heading_re.find_iter(markdown).count();
=======
    // Match user story patterns: **User Story:** or **User Story**:
    static USER_STORY_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?im)^\s*\*\*User\s+Story[:\*]").unwrap());
    summary.user_story_count = USER_STORY_RE.find_iter(markdown).count();

    // Match EARS-style acceptance criteria: WHEN ... THEN ... SHALL
    // Also match simpler patterns: GIVEN/WHEN/THEN or numbered criteria with SHALL
    static EARS_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?im)(WHEN\s+.+\s+THEN\s+.+\s+SHALL|GIVEN\s+.+\s+WHEN\s+.+\s+THEN)").unwrap()
    });
    summary.acceptance_criteria_count = EARS_RE.find_iter(markdown).count();

    // Match NFR patterns: **NFR-*, NFR-*, **Non-Functional*
    static NFR_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?im)(^\s*\*\*NFR[-\s]|\bNFR-\w+\b|^\s*\*\*Non-Functional)").unwrap());
    summary.nfr_count = NFR_RE.find_iter(markdown).count();

    // Match requirement headings: ### Requirement N or ## Requirement N (allow leading whitespace)
    static REQ_HEADING_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?im)^\s*#{2,3}\s+Requirement\s+\d+").unwrap());
    summary.requirement_count = REQ_HEADING_RE.find_iter(markdown).count();
>>>>>>> REPLACE
INNEREOF
