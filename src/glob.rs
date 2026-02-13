pub fn glob_match(pattern: &str, name: &str) -> bool {
    glob_match_bytes(pattern.as_bytes(), name.as_bytes())
}

fn glob_match_bytes(pat: &[u8], name: &[u8]) -> bool {
    match (pat.first(), name.first()) {
        (None, None) => true,
        (Some(&b'*'), _) => {
            glob_match_bytes(&pat[1..], name)
                || (!name.is_empty() && glob_match_bytes(pat, &name[1..]))
        }
        (Some(&b'?'), Some(_)) => glob_match_bytes(&pat[1..], &name[1..]),
        (Some(&p), Some(&n)) if p == n => glob_match_bytes(&pat[1..], &name[1..]),
        _ => false,
    }
}
