use crate::{pattern, pattern::Mode};
use bstr::{BString, ByteSlice};
use std::slice::Iter;
#[inline]
#[doc = " A sloppy parser that performs only the most basic checks, providing additional information"]
#[doc = " using `pattern::Mode` flags."]
#[doc = ""]
#[doc = " Returns `(pattern, mode, no_wildcard_len)`"]
pub fn pattern(mut pat: &[u8]) -> Option<(BString, pattern::Mode, Option<usize>)> {
    let mut mode = Mode::empty();
    if pat.is_empty() {
        return None;
    };
    if pat.first() == Some(&b'!') {
        mode |= Mode::NEGATIVE;
        pat = &pat[1..];
    } else if pat.first() == Some(&b'\\') {
        let second = pat.get(1);
        if second == Some(&b'!') || second == Some(&b'#') {
            pat = &pat[1..];
        }
    }
    if pat.iter().all(|b| b.is_ascii_whitespace()) {
        return None;
    }
    if pat.first() == Some(&b'/') {
        mode |= Mode::ABSOLUTE;
        pat = &pat[1..];
    }
    let mut pat = truncate_non_escaped_trailing_spaces(pat);
    if pat.last() == Some(&b'/') {
        mode |= Mode::MUST_BE_DIR;
        pat.pop();
    }
    if !pat.contains(&b'/') {
        mode |= Mode::NO_SUB_DIR;
    }
    if pat.first() == Some(&b'*') && first_wildcard_pos(&pat[1..]).is_none() {
        mode |= Mode::ENDS_WITH;
    }
    let pos_of_first_wildcard = first_wildcard_pos(&pat);
    Some((pat, mode, pos_of_first_wildcard))
}
fn first_wildcard_pos(pat: &[u8]) -> Option<usize> {
    pat.find_byteset(GLOB_CHARACTERS)
}
pub(crate) const GLOB_CHARACTERS: &[u8] = br"*?[\";
#[doc = " We always copy just because that's ultimately needed anyway, not because we always have to."]
fn truncate_non_escaped_trailing_spaces(buf: &[u8]) -> BString {
    match buf.rfind_not_byteset(br"\ ") {
        Some(pos) if pos + 1 == buf.len() => buf.into(),
        None => buf.into(),
        Some(start_of_non_space) => {
            let mut res: BString = buf[..start_of_non_space + 1].into();
            let mut trailing_bytes = buf[start_of_non_space + 1..].iter();
            let mut bare_spaces = 0;
            while let Some(b) = trailing_bytes.next() {
                bar(&mut res, &mut trailing_bytes, &mut bare_spaces, b)
            }
            res
        }
    }
}
fn bar(res: &mut BString, trailing_bytes: &mut Iter<'_, u8>, bare_spaces: &mut usize, b: &u8) {
    match b {
        b' ' => {
            (*bare_spaces) += 1;
        }
        b'\\' => {
            res.extend(std::iter::repeat(b' ').take((*bare_spaces)));
            (*bare_spaces) = 0;
            if trailing_bytes.next() == Some(&b' ') {
                res.push(b' ');
            }
        }
        _ => unreachable!("BUG: this must be either backslash or space"),
    }
}
