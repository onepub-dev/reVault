pub(crate) fn normalize(rules: &mut Vec<String>) {
    rules.sort();
    rules.dedup();
}

pub(crate) fn excluded(path: &str, rules: &[String]) -> bool {
    rules.iter().any(|rule| matches_rule(path, rule))
}

pub(crate) fn included(path: &str, rules: &[String]) -> bool {
    rules.is_empty() || rules.iter().any(|rule| matches_rule(path, rule))
}

fn matches_rule(path: &str, rule: &str) -> bool {
    let rule = rule.trim_start_matches("./").trim_end_matches('/');
    path == rule
        || path.starts_with(&format!("{rule}/"))
        || (!rule.contains('/')
            && glob_match_component(rule, path.rsplit('/').next().unwrap_or(path)))
        || glob_match_parts(
            &rule.split('/').collect::<Vec<_>>(),
            &path.split('/').collect::<Vec<_>>(),
        )
}

fn glob_match_parts(pattern: &[&str], text: &[&str]) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }
    if pattern[0] == "**" {
        return glob_match_parts(&pattern[1..], text)
            || (!text.is_empty() && glob_match_parts(pattern, &text[1..]));
    }
    !text.is_empty()
        && glob_match_component(pattern[0], text[0])
        && glob_match_parts(&pattern[1..], &text[1..])
}

fn glob_match_component(pattern: &str, text: &str) -> bool {
    let pattern = pattern.as_bytes();
    let text = text.as_bytes();
    let (mut p, mut t, mut star, mut matched) = (0, 0, None, 0);
    while t < text.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == text[t]) {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            matched = t;
            p += 1;
        } else if let Some(star_index) = star {
            p = star_index + 1;
            matched += 1;
            t = matched;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_rules_match_paths_and_globs() {
        assert!(included("src/main.rs", &["**/*.rs".to_string()]));
        assert!(excluded("target/debug/app", &["target/".to_string()]));
        assert!(!included("README.md", &["*.rs".to_string()]));
    }
}
