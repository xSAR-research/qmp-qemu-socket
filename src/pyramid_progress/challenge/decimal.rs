//! Checked finite native counter grammar after independent glyph classification.
//!
//! A fraction contains exactly one slash and two nonempty decimal integers.
//! Score alone permits correctly grouped thousands commas. No whitespace,
//! leading zeros, missing values, extra symbols or numeric overflow is accepted.

use super::{ChallengeKind, ChallengeProgress};


/// Maximum decimal places in either observed value; image bounds may be stricter.
const MAX_DIGITS: usize = 6;


/// Parse a classified bounded counter without allocation or inferred characters.
pub(super) fn parse(kind: ChallengeKind, symbols: &[u8]) -> Option<ChallengeProgress> {
    let slash = symbols.iter().position(|&symbol| symbol == b'/')?;
    let current = integer(&symbols[..slash], kind == ChallengeKind::Score)?;
    let target = integer(&symbols[slash + 1..], kind == ChallengeKind::Score)?;
    ChallengeProgress::new(kind, current, target)
}


/// Parse one bounded integer, checking comma grouping and arithmetic before use.
fn integer(symbols: &[u8], grouped: bool) -> Option<u32> {
    let first = *symbols.first()?;
    if !first.is_ascii_digit() || (first == b'0' && symbols.len() != 1) {
        return None;
    }
    let mut value = 0u32;
    let mut digits = 0usize;
    let mut group_digits = 0usize;
    let mut has_comma = false;
    for &symbol in symbols {
        if symbol.is_ascii_digit() {
            digits = digits.checked_add(1)?;
            group_digits = group_digits.checked_add(1)?;
            if digits > MAX_DIGITS || (has_comma && group_digits > 3) {
                return None;
            }
            value = value
                .checked_mul(10)?
                .checked_add(u32::from(symbol - b'0'))?;
        } else if symbol == b',' && grouped {
            if group_digits == 0 || group_digits > 3 || (has_comma && group_digits != 3) {
                return None;
            }
            group_digits = 0;
            has_comma = true;
        } else {
            return None;
        }
    }
    if group_digits == 0 || (has_comma && group_digits != 3) {
        return None;
    }
    Some(value)
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Recorded objective kinds retain their own current/target values and objective kind.
    #[test]
    fn actual_native_counter_strings_keep_objective_units() {
        for (kind, text, current, target) in [
            (ChallengeKind::Boards, b"0/2".as_slice(), 0, 2),
            (ChallengeKind::Boards, b"3/3".as_slice(), 3, 3),
            (ChallengeKind::Cards, b"4/12".as_slice(), 4, 12),
            (ChallengeKind::Cards, b"0/9".as_slice(), 0, 9),
            (ChallengeKind::Score, b"2,560/4,200".as_slice(), 2560, 4200),
            (ChallengeKind::Score, b"2,570/2,900".as_slice(), 2570, 2900),
            (ChallengeKind::Score, b"4,210/4,200".as_slice(), 4210, 4200),
        ] {
            assert_eq!(
                parse(kind, text),
                ChallengeProgress::new(kind, current, target)
            );
        }
    }


    /// Malformed, ambiguous, overlong and unobserved values are refused without repair.
    #[test]
    fn malformed_or_unbounded_counters_do_not_supply_progress() {
        for text in [
            "",
            "/",
            "1/",
            "/2",
            "1/2/3",
            "0/0",
            "01/2",
            "1/02",
            "1 /2",
            "1/ 2",
            "-1/2",
            "1.0/2",
            "1O/20",
            "1,2/3,000",
            "1,,000/2,000",
            "1,000,/2,000",
            "12,34/2,000",
            "1234,000/999,999",
            "1/1,00",
            "1/0,100",
            "1000000/999999",
            "999999999999/1",
        ] {
            assert_eq!(parse(ChallengeKind::Score, text.as_bytes()), None, "{text}");
        }
        assert_eq!(parse(ChallengeKind::Boards, b"3/2"), None);
        assert_eq!(parse(ChallengeKind::Boards, b"1,000/2,000"), None);
        assert_eq!(parse(ChallengeKind::Cards, b"13/12"), None);
    }


    /// Progress may increase during a route but never change the bound requirement.
    #[test]
    fn progress_requires_the_same_kind_target_and_monotonic_observation() {
        let original = parse(ChallengeKind::Cards, b"4/12").unwrap();
        let advanced = parse(ChallengeKind::Cards, b"5/12").unwrap();
        assert!(advanced.follows(original));
        assert!(original.follows(original));
        assert!(!original.follows(advanced));
        assert!(
            !parse(ChallengeKind::Boards, b"5/12")
                .unwrap()
                .follows(original)
        );
        assert!(
            !parse(ChallengeKind::Cards, b"5/13")
                .unwrap()
                .follows(original)
        );
        assert!(
            !parse(ChallengeKind::Score, b"5/12")
                .unwrap()
                .follows(original)
        );
        assert!(!original.target_reached());
        assert!(
            parse(ChallengeKind::Cards, b"12/12")
                .unwrap()
                .target_reached()
        );
        assert!(
            parse(ChallengeKind::Score, b"4,210/4,200")
                .unwrap()
                .target_reached()
        );
    }
}
