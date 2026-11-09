use frizbee::{Config, Matcher, Pattern};

pub fn patterns(query: &str) -> Vec<Pattern> {
    Pattern::parse_query(query)
        .into_iter()
        .map(|pattern| {
            let typos = (pattern.needle.chars().count() / 4).min(2) as u16;
            pattern.max_typos(Some(typos))
        })
        .collect()
}

pub fn match_indices(patterns: &[Pattern], haystacks: &[&str]) -> Vec<(usize, u16)> {
    if patterns.is_empty() || haystacks.is_empty() {
        return Vec::new();
    }

    Matcher::from_patterns(patterns, &Config::default())
        .match_list(haystacks)
        .into_iter()
        .map(|matched| (matched.index as usize, matched.score))
        .collect()
}
