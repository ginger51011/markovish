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

/// A borrowed version of [`Token`]; if [`Token`] is [`String`], then [`TokenRef`] is `&str`.
pub type TokenRef<'a> = &'a str;

/// A borrowed version of [`TokenPair`] that does not own its pair. Like [`TokenRef`] to [`Token`].
pub type TokenPairRef<'a> = (TokenRef<'a>, TokenRef<'a>);

impl<'a> From<&TokenPairRef<'a>> for TokenPair {
    fn from(value: &TokenPairRef) -> Self {
        Self(value.0.to_string(), value.1.to_string())
    }
}

impl TokenPair {
    pub fn as_ref(&self) -> TokenPairRef<'_> {
        (&self.0, &self.1)
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