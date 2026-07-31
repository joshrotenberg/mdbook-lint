//! Helpers for recognizing ATX heading delimiters.

/// Return the byte range of an unescaped trailing hash sequence, ignoring
/// spaces and tabs after it.
pub(crate) fn trailing_hash_sequence(line: &str) -> Option<(usize, usize)> {
    let end = line.trim_end_matches([' ', '\t']).len();
    let before_hashes = line[..end].trim_end_matches('#');
    let start = before_hashes.len();

    if start == end {
        return None;
    }

    // A backslash immediately before the sequence escapes its first hash, so
    // the run cannot be a CommonMark closing sequence.
    let backslash_count = line[..start]
        .chars()
        .rev()
        .take_while(|&ch| ch == '\\')
        .count();
    if backslash_count % 2 == 1 {
        return None;
    }

    Some((start, end))
}

/// Return the part of a line before a valid CommonMark closing hash sequence.
///
/// CommonMark requires the closing sequence to be preceded by a space or tab.
pub(crate) fn before_closing_hash_sequence(line: &str) -> Option<&str> {
    let (start, _) = trailing_hash_sequence(line)?;
    let preceding = line[..start].chars().next_back()?;

    if matches!(preceding, ' ' | '\t') {
        Some(&line[..start])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_commonmark_closing_sequences() {
        assert_eq!(
            before_closing_hash_sequence("### Heading ###"),
            Some("### Heading ")
        );
        assert_eq!(
            before_closing_hash_sequence("### Heading ###  "),
            Some("### Heading ")
        );
        assert_eq!(
            before_closing_hash_sequence("### Heading\t#"),
            Some("### Heading\t")
        );
    }

    #[test]
    fn rejects_content_hashes_and_escaped_hashes() {
        assert_eq!(before_closing_hash_sequence("### C#"), None);
        assert_eq!(before_closing_hash_sequence("### F##"), None);
        assert_eq!(before_closing_hash_sequence("### C\\#"), None);
        assert_eq!(before_closing_hash_sequence("### Heading"), None);
    }
}
