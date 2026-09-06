//! Optional fuzzy subsequence filter over workspace titles.

/// Result of a case-insensitive subsequence match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchResult {
    /// Higher is a tighter match (earlier + consecutive).
    pub score: i64,
    /// Character indices in the candidate that matched the query.
    pub positions: Vec<usize>,
}

/// Matches `query` as a case-insensitive subsequence of `candidate`.
pub fn fuzzy_match(candidate: &str, query: &str) -> Option<MatchResult> {
    if query.is_empty() {
        return Some(MatchResult {
            score: 0,
            positions: Vec::new(),
        });
    }

    let candidate_chars: Vec<char> = candidate.chars().collect();
    let query_chars: Vec<char> = query.chars().collect();
    if query_chars.len() > candidate_chars.len() {
        return None;
    }

    let mut positions = Vec::with_capacity(query_chars.len());
    let mut search_from = 0usize;
    for needle in &query_chars {
        let needle_lower = needle.to_lowercase().to_string();
        let found = candidate_chars
            .iter()
            .enumerate()
            .skip(search_from)
            .find(|(_, ch)| ch.to_lowercase().to_string() == needle_lower);
        match found {
            Some((idx, _)) => {
                positions.push(idx);
                search_from = idx + 1;
            }
            None => return None,
        }
    }

    Some(MatchResult {
        score: score_positions(&positions),
        positions,
    })
}

fn score_positions(positions: &[usize]) -> i64 {
    if positions.is_empty() {
        return 0;
    }
    let mut score = 100 - positions[0] as i64;
    let mut run = 1i64;
    for window in positions.windows(2) {
        if window[1] == window[0] + 1 {
            run += 1;
            score += run * 8;
        } else {
            run = 1;
            score -= (window[1] - window[0]) as i64;
        }
    }
    score
}

/// Filters rows by title, preserving original indices for activation.
pub fn filter_titles<T>(
    rows: &[T],
    query: &str,
    title: impl Fn(&T) -> &str,
) -> Vec<(usize, MatchResult)> {
    if query.is_empty() {
        return rows
            .iter()
            .enumerate()
            .map(|(idx, _)| {
                (
                    idx,
                    MatchResult {
                        score: 0,
                        positions: Vec::new(),
                    },
                )
            })
            .collect();
    }

    let mut matches: Vec<(usize, MatchResult)> = rows
        .iter()
        .enumerate()
        .filter_map(|(idx, row)| fuzzy_match(title(row), query).map(|m| (idx, m)))
        .collect();
    matches.sort_by(|a, b| b.1.score.cmp(&a.1.score).then(a.0.cmp(&b.0)));
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_matches_everything() {
        let result = fuzzy_match("accounting", "").unwrap();
        assert!(result.positions.is_empty());
        let filtered = filter_titles(&["a", "b"], "", |s| *s);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn subsequence_is_case_insensitive() {
        let result = fuzzy_match("Accounting", "act").unwrap();
        assert_eq!(result.positions, vec![0, 1, 6]);
    }

    #[test]
    fn rejects_non_subsequences() {
        assert!(fuzzy_match("inbox", "xyz").is_none());
        assert!(fuzzy_match("ab", "aa").is_none());
    }

    #[test]
    fn ranks_consecutive_matches_higher() {
        let tight = fuzzy_match("accounting", "acc").unwrap();
        let loose = fuzzy_match("a_c_c", "acc").unwrap();
        assert!(tight.score > loose.score);
    }
}
