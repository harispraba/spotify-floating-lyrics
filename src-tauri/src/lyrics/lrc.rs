//! Parser for the LRC lyrics format: `[mm:ss.xx]line text`.

use super::Line;

/// Parses LRC text into lines sorted by time. Handles multiple timestamps per
/// line, the `[offset:]` tag, and strips enhanced-LRC word timestamps
/// (`<mm:ss.xx>`). Empty lines are kept; they mark instrumental breaks.
pub fn parse(lrc: &str) -> Vec<Line> {
    let mut offset_ms: i64 = 0;
    let mut timed: Vec<(i64, String)> = Vec::new();

    for raw in lrc.lines() {
        let mut rest = raw.trim();
        let mut times = Vec::new();
        while rest.starts_with('[') {
            let Some(end) = rest.find(']') else { break };
            let tag = &rest[1..end];
            rest = &rest[end + 1..];
            if let Some(ms) = parse_timestamp(tag) {
                times.push(ms);
            } else if let Some(value) = tag.strip_prefix("offset:") {
                offset_ms = value.trim().parse().unwrap_or(0);
            }
        }
        if times.is_empty() {
            continue;
        }
        let text = strip_word_timestamps(rest);
        timed.extend(times.into_iter().map(|t| (t, text.clone())));
    }

    // A positive offset makes lyrics appear earlier.
    let mut lines: Vec<Line> = timed
        .into_iter()
        .map(|(t, text)| Line {
            time_ms: (t - offset_ms).max(0) as u64,
            text,
        })
        .collect();
    lines.sort_by_key(|line| line.time_ms);
    lines
}

/// Parses `mm:ss`, `mm:ss.x`, `mm:ss.xx`, `mm:ss.xxx` (or `:` before the fraction) into milliseconds.
fn parse_timestamp(tag: &str) -> Option<i64> {
    let (minutes, rest) = tag.trim().split_once(':')?;
    let minutes: i64 = minutes.parse().ok()?;
    let (seconds, fraction) = rest.split_once(['.', ':']).unwrap_or((rest, ""));
    let seconds: i64 = seconds.parse().ok()?;
    if !fraction.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let digits = &fraction[..fraction.len().min(3)];
    let fraction_ms = if digits.is_empty() {
        0
    } else {
        digits.parse::<i64>().ok()? * 10_i64.pow(3 - digits.len() as u32)
    };
    Some(minutes * 60_000 + seconds * 1000 + fraction_ms)
}

fn strip_word_timestamps(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('<') {
        match rest[start..].find('>') {
            Some(len) if parse_timestamp(&rest[start + 1..start + len]).is_some() => {
                out.push_str(&rest[..start]);
                out.push(' ');
                rest = &rest[start + len + 1..];
            }
            _ => {
                out.push_str(&rest[..=start]);
                rest = &rest[start + 1..];
            }
        }
    }
    out.push_str(rest);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(time_ms: u64, text: &str) -> Line {
        Line {
            time_ms,
            text: text.to_string(),
        }
    }

    #[test]
    fn parses_timestamps_with_any_precision() {
        let lines = parse("[00:01]one\n[00:02.5]two\n[00:03.25]three\n[01:04.125]four");
        assert_eq!(
            lines,
            vec![
                line(1000, "one"),
                line(2500, "two"),
                line(3250, "three"),
                line(64125, "four")
            ]
        );
    }

    #[test]
    fn expands_repeated_lines_and_sorts() {
        let lines = parse("[00:10.00][00:30.00]chorus\n[00:20.00]verse");
        assert_eq!(
            lines,
            vec![
                line(10000, "chorus"),
                line(20000, "verse"),
                line(30000, "chorus")
            ]
        );
    }

    #[test]
    fn ignores_metadata_and_applies_offset() {
        let lines = parse(
            "[ar:Someone]\n[ti:Song]\n[length: 03:25]\n[offset:+500]\n[00:01.00]hello\n[00:00.20]early",
        );
        assert_eq!(lines, vec![line(0, "early"), line(500, "hello")]);
    }

    #[test]
    fn strips_word_timestamps() {
        let lines = parse("[00:01.00]<00:01.00>Hello <00:01.50>world <00:02.00>");
        assert_eq!(lines, vec![line(1000, "Hello world")]);
    }

    #[test]
    fn keeps_angle_brackets_that_are_not_timestamps() {
        let lines = parse("[00:01.00]a <3 b <tag>");
        assert_eq!(lines, vec![line(1000, "a <3 b <tag>")]);
    }

    #[test]
    fn keeps_empty_lines_and_skips_untimed_ones() {
        let lines = parse("just text\n[00:01.00]one\n[00:05.00] \n\n[00:06.00]two");
        assert_eq!(
            lines,
            vec![line(1000, "one"), line(5000, ""), line(6000, "two")]
        );
    }

    #[test]
    fn handles_crlf_and_colon_fraction() {
        let lines = parse("[00:01:50]one\r\n[00:02.00]two\r\n");
        assert_eq!(lines, vec![line(1500, "one"), line(2000, "two")]);
    }
}
