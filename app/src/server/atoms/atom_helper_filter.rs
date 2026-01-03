use regex::Regex;

pub fn should_filter_path(rel: &str, regexes: &[Regex]) -> bool {
    let rel = rel.replace('\\', "/");
    regexes.iter().any(|re| re.is_match(&rel))
}
