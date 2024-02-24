//! Dead simple text generation using markov chains. The text generator behind [`pandoras_pot`](https://github.com/ginger51011/pandoras_pot/).
//!
//! Right now this generator only supports second order Markov chains, that is, it looks at two
//! symbols at a time and then guesses what the third might be (weighted depending on how likely
//! the three-word combination is in the source text). The randomness is built using a weighted
//! distribution (see [`rand_distr::weighted_alias::WeightedAliasIndex`]).
//!
//! # Features
//!
//! `markovish` comes with some feature(s) that you can disable (or enable) at will. They are:
//!
//! - `hashbrown` - Uses the [`hashbrown`](https://crates.io/crates/hashbrown) crate for the
//! internal chain. While a lot faster, it is not as protected against HashDoS attacks. Enabled by
//! default.

#[cfg(feature = "hashbrown")]
use hashbrown::Equivalent;
#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;

use rand::{seq::IteratorRandom, Rng};

#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;

use itertools::Itertools;
use rand_distr::{weighted_alias::WeightedAliasIndex, Distribution};
use unicode_segmentation::UnicodeSegmentation;

/// At the heart of a [`Chain`] is a [`Token`]. In fact, this is just a String. But we make a
/// distinction here: A Token is the output of
/// [`unicode_segmentation::UnicodeSegmentation::split_word_bounds()`]; that is, a word, a symbol
/// like `"`, or something else. See that crate for more information.
pub type Token = String;

/// An owned pair of [`Token`]s.
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

#[cfg(feature = "hashbrown")]
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

    pub fn get_random_token(&self, rng: &mut impl Rng) -> &Token {
        &self.choices[self.dist.sample(rng)]
    }
}

/// Builder for [`TokenDistribution`]. Used when parsing a text to add a lot of words, and then to
/// build a list of [`TokenDistribution`] using how many times they appeared.
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
        match self.map.get_mut(token) {
            Some(n) => {
                *n += 1;
            }
            None => {
                self.map.insert(token.to_string(), 1);
            }
        }
    }
}

impl Default for TokenDistributionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple second order Markov chain. This chain might behave in ways you do not expect; Since we
/// are looking at [`Token`]s, and not words.
///
/// ```
/// # use markovish::{Chain, ChainBuilder};
/// # use rand::thread_rng;
/// let mut cb = Chain::builder();
/// cb.feed_str("I am &str").unwrap();
/// let chain = cb.build();
///
/// // You would expect this to be "&str", but no!
/// assert_eq!(
///     chain.generate_next_token(&mut thread_rng(), &("I", "am")).as_deref(),
///     None
/// );
///
/// // We have a space which is a token!
/// assert_eq!(
///     chain.generate_next_token(&mut thread_rng(), &("I", " ")).as_deref(),
///     Some(&"am".to_string())
/// );
/// ```
#[derive(Clone, Debug)]
pub struct Chain {
    map: HashMap<TokenPair, TokenDistribution>,
}

impl Chain {
    pub fn builder() -> ChainBuilder {
        ChainBuilder::new()
    }

    /// Generates a random new token using the previous tokens.
    ///
    /// If the chain has never seen the `prev` tokens together, `None` is returned.
    pub fn generate_next_token(&self, rng: &mut impl Rng, prev: &(&str, &str)) -> Option<&Token> {
        let dist = self.map.get(prev)?;
        Some(dist.get_random_token(rng))
    }

    /// Randomly chooses two tokens that are known to be able to generate a new token. If no
    /// start tokens exist, `None` is returned.
    pub fn start_tokens(&self, rng: &mut impl Rng) -> Option<&TokenPair> {
        self.map.keys().choose(rng)
    }

    /// Generates `n` tokens, using previously used tokens to generate new ones. If two tokens are found that have never been seen before,
    /// two new starting tokens are generated using [`Chain::start_tokens()`].
    ///
    /// If the chain has never seen the `prev` tokens together, `None` is returned.
    ///
    /// # Panics
    ///
    /// Will panic if `n` is so big no vector can hold that many elements.
    pub fn generate_n_tokens(
        &self,
        rng: &mut impl Rng,
        prev: &(&str, &str),
        n: usize,
    ) -> Option<Vec<&Token>> {
        if n < 1 {
            return Some(Vec::new());
        }

        // We first make sure the `prev` tokens have ever been seen together before
        // allocating the result
        let first = self.generate_next_token(rng, prev)?;
        let mut res = Vec::with_capacity(n);

        res.push(first);

        let (mut left, mut right) = (prev.1, first);

        // Since we are not including n, we don't take (n - 1)
        while res.len() < n {
            if let Some(next) = self.generate_next_token(rng, &(&left, &right)) {
                res.push(next);
                left = right;
                right = next;
            } else {
                // We found two tokens that have never been seen together, we have to get new start
                // tokens. Unwrap is safe, since we could never get this far without any start
                // tokens.
                let tp = self.start_tokens(rng).unwrap();

                // Figure out if we have room for both
                let r = n - res.len();
                if r >= 2 {
                    left = &tp.0;
                    right = &tp.1;
                    res.push(&tp.0);
                    res.push(&tp.1);
                } else if r == 1 {
                    res.push(&tp.0);
                    break;
                } else {
                    // Should never happen
                    break;
                }
            }
        }

        Some(res)
    }

    /// Generates `n` tokens, using previously used tokens to generate new ones. Less tokens may
    /// be generated, if two tokens are found that have never been seen before.
    ///
    /// If the chain has never seen the `prev` tokens together, `None` is returned.
    ///
    /// # Panics
    ///
    /// Will panic if `n` is so big no vector can hold that many elements.
    pub fn generate_max_n_tokens(
        &self,
        rng: &mut impl Rng,
        prev: &(&str, &str),
        n: usize,
    ) -> Option<Vec<&Token>> {
        if n < 1 {
            return Some(Vec::new());
        }

        // We first make sure the `prev` tokens have ever been seen together before
        // allocating the result
        let first = self.generate_next_token(rng, prev)?;
        let mut res = Vec::with_capacity(n);

        res.push(first);
        let remaining = n - 1;

        let (mut left, mut right) = (prev.1, first);

        for _ in 0..remaining {
            if let Some(next) = self.generate_next_token(rng, &(&left, &right)) {
                res.push(next);
                left = right;
                right = next;
            } else {
                // We found two tokens that have never been seen together
                break;
            }
        }

        Some(res)
    }
}

/// Builds a Chain by being fed strings and keeping track of the likelihood that one token
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

#[cfg(test)]
mod tests {
    use rand::thread_rng;

    use crate::{Chain, TokenDistribution};

    #[test]
    #[should_panic]
    fn empty_chain_builder_panics() {
        let _ = Chain::builder().build();
    }

    #[test]
    #[should_panic]
    fn empty_token_dist_builder_panics() {
        let _ = TokenDistribution::builder().build();
    }

    #[test]
    fn simple_single_possible_token() {
        let s = "I am";
        let mut cb = Chain::builder();
        cb.feed_str(s).unwrap();
        let chain = cb.build();
        assert_eq!(
            chain
                .generate_next_token(&mut thread_rng(), &("I", " "))
                .unwrap(),
            "am"
        );
    }

    #[test]
    fn simple_single_impossible_token() {
        let s = "I am";
        let mut cb = Chain::builder();
        cb.feed_str(s).unwrap();
        let chain = cb.build();
        assert!(chain
            .generate_next_token(&mut thread_rng(), &("You", " "))
            .is_none());
    }

    #[test]
    fn simple_generate_max_n_tokens() {
        let s = "I am-full!of?cats";
        let mut cb = Chain::builder();
        cb.feed_str(s).unwrap();
        let chain = cb.build();

        assert_eq!(
            chain
                .generate_max_n_tokens(&mut thread_rng(), &("I", " "), 7)
                .unwrap(),
            vec!["am", "-", "full", "!", "of", "?", "cats"],
        );

        // Now with an actual limit
        assert_eq!(
            chain
                .generate_max_n_tokens(&mut thread_rng(), &("I", " "), 2)
                .unwrap(),
            vec!["am", "-"],
        );

        // Now with extra
        assert_eq!(
            chain
                .generate_max_n_tokens(&mut thread_rng(), &("I", " "), 13)
                .unwrap()
                .len(),
            7
        );
    }

    #[test]
    fn simple_generate_n_tokens() {
        let s = "I am-full!of?cats";
        let mut cb = Chain::builder();
        cb.feed_str(s).unwrap();
        let chain = cb.build();
        assert_eq!(
            chain
                .generate_n_tokens(&mut thread_rng(), &("I", " "), 7)
                .unwrap(),
            vec!["am", "-", "full", "!", "of", "?", "cats"],
        );

        // Now with an actual limit
        assert_eq!(
            chain
                .generate_n_tokens(&mut thread_rng(), &("I", " "), 2)
                .unwrap(),
            vec!["am", "-"],
        );

        // Now with extra
        assert_eq!(
            chain
                .generate_n_tokens(&mut thread_rng(), &("I", " "), 13)
                .unwrap()
                .len(),
            13
        );

        // Exactly on the line, so only one of the new start tokens should be taken
        assert_eq!(
            chain
                .generate_n_tokens(&mut thread_rng(), &("I", " "), 8)
                .unwrap()
                .len(),
            8
        );
    }

    #[test]
    fn simple_generate_max_n_tokens_zero() {
        let s = "I am-full!of?cats";
        let mut cb = Chain::builder();
        cb.feed_str(s).unwrap();
        let chain = cb.build();
        assert!(chain
            .generate_max_n_tokens(&mut thread_rng(), &("I", " "), 0)
            .unwrap()
            .is_empty())
    }

    #[test]
    fn simple_generate_max_n_tokens_impossible_first() {
        let s = "I am-full!of?cats";
        let mut cb = Chain::builder();
        cb.feed_str(s).unwrap();
        let chain = cb.build();
        assert!(chain
            .generate_max_n_tokens(&mut thread_rng(), &("You", " "), 13)
            .is_none())
    }

    #[test]
    fn simple_generate_n_tokens_zero() {
        let s = "I am-full!of?cats";
        let mut cb = Chain::builder();
        cb.feed_str(s).unwrap();
        let chain = cb.build();
        assert!(chain
            .generate_n_tokens(&mut thread_rng(), &("I", " "), 0)
            .unwrap()
            .is_empty())
    }

    #[test]
    fn simple_generate_n_tokens_impossible_first() {
        let s = "I am-full!of?cats";
        let mut cb = Chain::builder();
        cb.feed_str(s).unwrap();
        let chain = cb.build();
        assert!(chain
            .generate_n_tokens(&mut thread_rng(), &("You", " "), 13)
            .is_none())
    }
}
