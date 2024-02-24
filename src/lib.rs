//! Dead simple text generation using markov chains.
//!
//! Right now this generator only supports second order Markov chains, that is, it looks at two
//! symbols at a time and then guesses what the third might be (weighted depending on how likely
//! the three-word combination is in the source text). The randomness is built using a weighted
//! distribution (see [`rand_distr::weighted_alias::WeightedAliasIndex`]).
//!
//! # Features
//!
//! `markovish` comes with some features that you can disable (or enable) at will. They are:
//!
//! - `small_rng` - Uses the `rand` feature `small_rng` for randomness. Should be faster, but not
//! as random perhaps.
//! - `hashbrown` - Uses the [`hashbrown`](https://crates.io/crates/hashbrown) crate for the
//! internal chain. While a lot faster, it is not as protected against HashDoS attacks. Enabled by
//! default.

use hashbrown::Equivalent;
#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;

#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;

use itertools::Itertools;
use rand_distr::weighted_alias::WeightedAliasIndex;
use unicode_segmentation::UnicodeSegmentation;

/// At the heart of a markovish Chain is a [`Token`]. In fact, this is just a String. But we make a
/// distinction here: A Token is the output of
/// [`unicode_segmentation::UnicodeSegmentation::split_word_bounds()`]; that is, a word, a symbol
/// like `"`, or something else. See that crate for more information.
type Token = String;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenPair(Token, Token);

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

/// A distribution of choices and their likelyhood.
#[derive(Clone, Debug)]
pub struct TokenDistribution {
    /// Mappings of index in choices to their likelyhood.
    dist: WeightedAliasIndex<usize>,
    /// The actual choices
    choices: Vec<Token>,
}

impl TokenDistribution {
    pub fn builder() -> TokenDistributionBuilder {
        TokenDistributionBuilder::new()
    }
}

/// Builder for [`LikelyTokenList`]. Used when parsing a text to add a lot of words, and then to
/// build a list of [`LikelyToken`] using how many times they appeared.
#[derive(Clone, Debug)]
pub struct TokenDistributionBuilder {
    /// Counts how many times a token is likely to appear.
    map: HashMap<String, usize>,
}

impl TokenDistributionBuilder {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Creates a weighted distribution for the likelyhood of tokens to appear.
    ///
    /// # Panics
    ///
    /// Will panic if:
    ///
    /// - There are no inserted tokens
    pub fn build(self) -> TokenDistribution {
        let mut choices = Vec::with_capacity(self.map.len());
        let mut occurances = Vec::with_capacity(self.map.len());
        for (token, n) in self.map {
            choices.push(token);
            occurances.push(n);
        }

        TokenDistribution {
            dist: WeightedAliasIndex::new(occurances)
                .expect("failed to create weighted alias index"),
            choices,
        }
    }

    /// Add an occurance of this token.
    pub fn add_token(&mut self, token: &str) {
        let mut default = 0_usize;
        let n = self.map.get_mut(token).unwrap_or(&mut default);
        *n += 1;
    }
}

impl Default for TokenDistributionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple second order Markov chain.
#[derive(Clone, Debug)]
pub struct Chain {
    map: HashMap<TokenPair, TokenDistribution>,
}

impl Chain {
    pub fn builder() -> ChainBuilder {
        ChainBuilder::new()
    }
}

/// Builds a Chain map by being fed strings and keeping track of the likelyhood that one token
/// follows two others.
#[derive(Clone, Debug)]
pub struct ChainBuilder {
    map: HashMap<TokenPair, TokenDistributionBuilder>,
}

impl ChainBuilder {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Uses up the builder and creates a new chain.
    ///
    /// # Panics
    ///
    /// If the builder has not been fed any strings.
    pub fn build(self) -> Chain {
        assert!(
            !self.map.is_empty(),
            "the builder has not been fed any strings"
        );
        let mut chain_map = HashMap::with_capacity(self.map.len());
        for (pair, dist_builder) in self.map {
            chain_map.insert(pair, dist_builder.build());
        }

        Chain { map: chain_map }
    }

    /// Add the occurance of `next` following `prev`.
    pub fn add_occurance(&mut self, prev: (&str, &str), next: &str) {
        match self.map.get_mut(&prev) {
            Some(b) => {
                b.add_token(next);
            }
            None => {
                let mut b = TokenDistributionBuilder::new();
                b.add_token(next);
                let tp = TokenPair::from(prev);
                self.map.insert(tp, b);
            }
        }
    }

    /// Feeds the chain builder with more text, adding the tokens in this string to the mappings of
    /// this. May fail if the input string is too short.
    ///
    /// The tokens are from [`unicode_segmentation::UnicodeSegmentation::split_word_bounds()`].
    pub fn feed_str(&mut self, content: &str) -> Result<(), String> {
        let tokens = content.split_word_bounds();

        for (right, left, next) in tokens.tuple_windows() {
            self.add_occurance((right, left), next);
        }

        Ok(())
    }
}

impl Default for ChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}
