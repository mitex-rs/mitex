use core::fmt;

use crate::{argument_kind::ARGUMENT_KIND_TERM, ArgPattern};
use mitex_glob::glob_match_prefix;
use mitex_spec::GlobStr;

/// A matcher for arguments of a TeX command
/// It is created by `ArgMatcherBuilder`
pub enum ArgMatcher {
    /// None of arguments is passed, i.e. it is processed as a
    /// variable in typst.
    /// Note: this is different from FixedLenTerm(0)
    /// Where, \alpha is None, but not FixedLenTerm(0)
    /// E.g. \alpha => $alpha$
    None,
    /// Fixed or Range length pattern, equivalent to `/t{0,x}/g`
    /// E.g. \hat x y => $hat(x) y$,
    /// E.g. 1 \sum\limits => $1 limits(sum)$,
    AtMostTerm { max: u8, counter: u8 },
    /// Receive terms as much as possible,
    /// equivalent to `/t*/g`
    /// E.g. \over, \displaystyle
    Greedy,
    /// Most powerful pattern, but slightly slow
    /// Note that the glob must accept all prefix of the input
    ///
    /// E.g. \sqrt has a glob pattern of `{,b}t`
    /// Description:
    /// - {,b}: first, it matches an bracket option, e.g. `\sqrt[3]`
    /// - t: it later matches a single term, e.g. `\sqrt[3]{a}` or `\sqrt{a}`
    ///
    /// Note: any prefix of the glob is valid in parse stage hence you need to
    /// check whether it is complete in later stage.
    Glob { re: GlobStr, prefix: String },
}

impl ArgMatcher {
    /// Check if the matcher is greedy
    pub fn is_greedy(&self) -> bool {
        matches!(self, Self::Greedy)
    }

    /// Check if the matcher is ending match with that char
    ///
    /// Return true if modified as term
    pub fn match_as_term(&mut self, text: char) -> Option<bool> {
        match self {
            Self::None => None,
            Self::Greedy => Some(text != ARGUMENT_KIND_TERM),
            Self::AtMostTerm { .. } => self
                .try_match(ARGUMENT_KIND_TERM)
                .then_some(text != ARGUMENT_KIND_TERM),
            Self::Glob { .. } => self.try_match(text).then_some(false),
        }
    }

    /// Check if the matcher is ending match with that char
    pub fn try_match(&mut self, text: char) -> bool {
        match self {
            Self::None => false,
            Self::Greedy => true,
            Self::AtMostTerm { ref max, counter } => {
                // println!("try match {} {}, {}", text, self.counter, max);
                if text != ARGUMENT_KIND_TERM {
                    return false;
                }
                let ct = *counter < *max;
                *counter += 1;
                ct
            }
            Self::Glob { ref re, prefix } => {
                prefix.push(text);
                glob_match_prefix(&re.0, prefix)
            }
        }
    }
}

#[derive(Default)]
pub struct ArgMatcherBuilder {}

impl fmt::Debug for ArgMatcherBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArgMatcherBuilder").finish()
    }
}

impl ArgMatcherBuilder {
    pub fn start_match(&mut self, pat_meta: &ArgPattern) -> ArgMatcher {
        match pat_meta {
            ArgPattern::None => ArgMatcher::None,
            ArgPattern::RangeLenTerm { max: mx, .. } | ArgPattern::FixedLenTerm { len: mx } => {
                if mx == &0 {
                    ArgMatcher::None
                } else {
                    ArgMatcher::AtMostTerm {
                        max: *mx,
                        counter: 0,
                    }
                }
            }
            ArgPattern::Greedy => ArgMatcher::Greedy,
            ArgPattern::Glob { pattern: re } => ArgMatcher::Glob {
                re: re.clone(),
                prefix: String::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use mitex_glob::glob_match_prefix;

    #[test]
    fn glob_prefix() {
        assert!(glob_match_prefix("abc", ""));
        assert!(glob_match_prefix("abc", "a"));
        assert!(glob_match_prefix("abc", "ab"));
        assert!(glob_match_prefix("abc", "abc"));
        assert!(!glob_match_prefix("abc", "b"));
        assert!(!glob_match_prefix("abc", "ac"));
        assert!(!glob_match_prefix("abc", "abd"));
        assert!(!glob_match_prefix("abc", "abcd"));
        assert!(!glob_match_prefix("abc", "abca"));
    }

    #[test]
    fn glob_negated_pattern() {
        assert!(!glob_match_prefix("!", ""));
        assert!(!glob_match_prefix("!abc", ""));
        assert!(!glob_match_prefix("!abc", "a"));
        assert!(!glob_match_prefix("!abc", "ab"));
        assert!(!glob_match_prefix("!abc", "abc"));
        assert!(glob_match_prefix("!abc", "b"));
        assert!(glob_match_prefix("!abc", "ac"));
        assert!(glob_match_prefix("!abc", "abd"));
        assert!(glob_match_prefix("!abc", "abcd"));
        assert!(glob_match_prefix("!abc", "abca"));
    }

    #[test]
    fn glob_sqrt() {
        assert!(glob_match_prefix("{,b}t", "t"));
        assert!(glob_match_prefix("{,b}t", ""));
        assert!(glob_match_prefix("{,b}t", "b"));
        assert!(glob_match_prefix("{,b}t", "bt"));
        assert!(!glob_match_prefix("{,b}t", "tt"));
    }
}
