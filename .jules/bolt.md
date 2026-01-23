## 2024-03-21 - [GlobSet Optimization]
**Learning:** Checking a file path against multiple `GlobSet`s sequentially is less efficient than combining them into a single `GlobSet` and checking match indices. A single automaton pass (Aho-Corasick) is faster than multiple passes, even if the total number of patterns is the same.
**Action:** When classifying strings against multiple disjoint sets of patterns, combine them into one `GlobSet` (or `RegexSet`) and use the match index to map back to the classification.
