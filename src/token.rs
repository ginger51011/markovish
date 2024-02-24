//! At the heart of a [`Chain`](crate::Chain) is a [`Token`]. In fact, this is just a String. But we make a
//! distinction here: A Token is the output of
//! [`unicode_segmentation::UnicodeSegmentation::split_word_bounds()`]; that is, it can be a word, a symbol
//! like `"`, or something else. See that crate for more information.

use hashbrown::Equivalent;

/// Representation of a string segment.
pub type Token = String;

/// An owned pair of [`Token`]s.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenPair(pub Token, pub Token);

impl From<(&str, &str)> for TokenPair {
    fn from(value: (&str, &str)) -> Self {
        Self(value.0.to_string(), value.1.to_string())
    }
}

impl PartialEq<(&str, &str)> for TokenPair {
    fn eq(&self, other: &(&str, &str)) -> bool {
        self.0 == *other.0 && self.1 == *other.1
    }
}

impl Equivalent<TokenPair> for (&str, &str) {
    fn equivalent(&self, key: &TokenPair) -> bool {
        key.eq(self)
    }
}
