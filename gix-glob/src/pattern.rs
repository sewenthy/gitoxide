use crate::{pattern, wildmatch, Pattern};
use bitflags::bitflags;
use bstr::{BStr, ByteSlice};
use std::fmt;
bitflags! { # [doc = " Information about a [`Pattern`]."] # [doc = ""] # [doc = " Its main purpose is to accelerate pattern matching, or to negate the match result or to"] # [doc = " keep special rules only applicable when matching paths."] # [doc = ""] # [doc = " The mode is typically created when parsing the pattern by inspecting it and isn't typically handled by the user."] # [cfg_attr (feature = "serde1" , derive (serde :: Serialize , serde :: Deserialize))] pub struct Mode : u32 { # [doc = " The pattern does not contain a sub-directory and - it doesn't contain slashes after removing the trailing one."] const NO_SUB_DIR = 1 << 0 ; # [doc = " A pattern that is '*literal', meaning that it ends with what's given here"] const ENDS_WITH = 1 << 1 ; # [doc = " The pattern must match a directory, and not a file."] const MUST_BE_DIR = 1 << 2 ; # [doc = " The pattern matches, but should be negated. Note that this mode has to be checked and applied by the caller."] const NEGATIVE = 1 << 3 ; # [doc = " The pattern starts with a slash and thus matches only from the beginning."] const ABSOLUTE = 1 << 4 ; } }
#[doc = " Describes whether to match a path case sensitively or not."]
#[doc = ""]
#[doc = " Used in [Pattern::matches_repo_relative_path()]."]
#[derive(Debug, PartialOrd, PartialEq, Copy, Clone, Hash, Ord, Eq)]
pub enum Case {
    #[doc = " The case affects the match"]
    Sensitive,
    #[doc = " Ignore the case of ascii characters."]
    Fold,
}
impl Default for Case {
    fn default() -> Self {
        Case::Sensitive
    }
}
impl Pattern {
    #[doc = " Parse the given `text` as pattern, or return `None` if `text` was empty."]
    pub fn from_bytes(text: &[u8]) -> Option<Self> {
        crate::parse::pattern(text).map(|(text, mode, first_wildcard_pos)| Pattern {
            text,
            mode,
            first_wildcard_pos,
        })
    }
    #[doc = " Return true if a match is negated."]
    pub fn is_negative(&self) -> bool {
        self.mode.contains(Mode::NEGATIVE)
    }
    #[doc = " Match the given `path` which takes slashes (and only slashes) literally, and is relative to the repository root."]
    #[doc = " Note that `path` is assumed to be relative to the repository."]
    #[doc = ""]
    #[doc = " We may take various shortcuts which is when `basename_start_pos` and `is_dir` come into play."]
    #[doc = " `basename_start_pos` is the index at which the `path`'s basename starts."]
    #[doc = ""]
    #[doc = " Lastly, `case` folding can be configured as well."]
    pub fn matches_repo_relative_path<'a>(
        &self,
        path: impl Into<&'a BStr>,
        basename_start_pos: Option<usize>,
        is_dir: Option<bool>,
        case: Case,
    ) -> bool {
        let is_dir = is_dir.unwrap_or(false);
        if !is_dir && self.mode.contains(pattern::Mode::MUST_BE_DIR) {
            return false;
        }
        let flags = Self::bar(case);
        let path = path.into();
        debug_assert_eq!(
            basename_start_pos,
            path.rfind_byte(b'/').map(|p| p + 1),
            "BUG: invalid cached basename_start_pos provided"
        );
        debug_assert!(!path.starts_with(b"/"), "input path must be relative");
        if self.mode.contains(pattern::Mode::NO_SUB_DIR)
            && !self.mode.contains(pattern::Mode::ABSOLUTE)
        {
            let basename = &path[basename_start_pos.unwrap_or_default()..];
            self.matches(basename, flags)
        } else {
            self.matches(path, flags)
        }
    }
    fn bar(case: Case) -> wildmatch::Mode {
        wildmatch::Mode::NO_MATCH_SLASH_LITERAL
            | match case {
                Case::Fold => wildmatch::Mode::IGNORE_CASE,
                Case::Sensitive => wildmatch::Mode::empty(),
            }
    }
    #[doc = " See if `value` matches this pattern in the given `mode`."]
    #[doc = ""]
    #[doc = " `mode` can identify `value` as path which won't match the slash character, and can match"]
    #[doc = " strings with cases ignored as well. Note that the case folding performed here is ASCII only."]
    #[doc = ""]
    #[doc = " Note that this method uses some shortcuts to accelerate simple patterns."]
    fn matches<'a>(&self, value: impl Into<&'a BStr>, mode: wildmatch::Mode) -> bool {
        let value = value.into();
        match self.first_wildcard_pos {
            Some(pos) if self.mode.contains(pattern::Mode::ENDS_WITH) && !value.contains(&b'/') => {
                let text = &self.text[pos + 1..];
                if mode.contains(wildmatch::Mode::IGNORE_CASE) {
                    value
                        .len()
                        .checked_sub(text.len())
                        .map(|start| text.eq_ignore_ascii_case(&value[start..]))
                        .unwrap_or(false)
                } else {
                    value.ends_with(text.as_ref())
                }
            }
            Some(pos) => {
                if mode.contains(wildmatch::Mode::IGNORE_CASE) {
                    if !value
                        .get(..pos)
                        .map_or(false, |value| value.eq_ignore_ascii_case(&self.text[..pos]))
                    {
                        return false;
                    }
                } else if !value.starts_with(&self.text[..pos]) {
                    return false;
                }
                crate::wildmatch(self.text.as_bstr(), value, mode)
            }
            None => {
                if mode.contains(wildmatch::Mode::IGNORE_CASE) {
                    self.text.eq_ignore_ascii_case(value)
                } else {
                    self.text == value
                }
            }
        }
    }
}
impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.mode.contains(Mode::NEGATIVE) {
            "!".fmt(f)?;
        }
        if self.mode.contains(Mode::ABSOLUTE) {
            "/".fmt(f)?;
        }
        self.text.fmt(f)?;
        if self.mode.contains(Mode::MUST_BE_DIR) {
            "/".fmt(f)?;
        }
        Ok(())
    }
}
