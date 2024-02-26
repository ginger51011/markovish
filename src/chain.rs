//! See the top level crate documentation for information about the [`Chain`] type.

use hashbrown::HashMap;

use itertools::Itertools;
use rand::seq::IteratorRandom;
use rand::Rng;
use unicode_segmentation::UnicodeSegmentation;

use crate::distribution::{TokenDistribution, TokenDistributionBuilder};
use crate::token::{TokenPair, TokenPairRef, TokenRef};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Simple second order Markov chain. This chain might behave in ways you do not expect; Since we
/// are looking at [`Token`](crate::token::Token)s, and not words. If this is not desired, you
/// can use your own splitting of tokens and use [`ChainBuilder::feed_tokens()`].
///
/// ```
/// # use markovish::{Chain, ChainBuilder};
/// # use rand::thread_rng;
/// use markovish::IntoChainBuilder;
///
/// // You can use `.into_cb()` for the result of `feed_*` methods. This way, you can
/// // ignore if the feed was successfull (enough tokens were provided) or not.
/// let chain = Chain::builder().feed_str("I am &str").into_cb().build().unwrap();
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
///     Some("am")
/// );
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Chain {
    map: HashMap<TokenPair, TokenDistribution>,
}
impl Chain {
    /// Creates a new second order Markov chain from a string.
    ///
    /// If the provided text is not long enough to create a [`Chain`],
    /// an empty [`ChainBuilder`] is returned instead.
    pub fn from_text(content: &str) -> Result<Self, ChainBuilder> {
        let mut cb = Self::builder();
        cb = cb.feed_str(content)?.into();
        cb.build()
    }

    pub fn builder() -> ChainBuilder {
        ChainBuilder::new()
    }

    /// Returns an iterator of all pairs that have been found in the source text(s). When calling
    /// [`Chain::start_tokens()`], a [`TokenPair`] is randomly chosen from this list.
    ///
    /// This can be used together with [`Chain::generate_max_n_tokens()`] to get more fine-grained
    /// control of how the chain is restarted if it stumbles on a token pair with no possible next
    /// token. You can filter the pairs so that they are more likely to start a sentence.
    ///
    /// # Examples
    ///
    /// ```
    /// # use markovish::Chain;
    /// let chain = Chain::from_text("I am but a tiny example! I have three sentences. U?").unwrap();
    /// let good_starting_points: Vec<_> = chain.pairs()
    ///                                         .filter(|tp| tp.0.as_str() == "." || tp.0.as_str() == "!")
    ///                                         .collect();
    /// assert_eq!(good_starting_points.len(), 2);
    /// ```
    pub fn pairs(&self) -> impl Iterator<Item = &TokenPair> {
        self.map.keys()
    }

    /// Randomly chooses two tokens that are known to be able to generate a new token. If no
    /// start tokens exist, `None` is returned.
    ///
    /// While this is an easy way, the returned value can be any two pairs of token in
    /// the source text. If you need more control, you could first filter on [`Chain::pairs()`],
    /// and then randomly choose starting tokens from that subset.
    pub fn start_tokens(&self, rng: &mut impl Rng) -> Option<&TokenPair> {
        self.pairs().choose(rng)
    }

    /// Generates a string with `n` tokens, randomly choosing a starting point.
    ///
    /// # Examples
    /// ```
    /// # let s = "I am an example string hello I very cool";
    /// ```
    pub fn generate_str(&self, rng: &mut impl Rng, n: usize) -> Option<Vec<&str>> {
        let start = self.start_tokens(rng)?;
        self.generate_n_tokens(rng, &start.as_ref(), n)
    }

    /// Generates a random new token using the previous tokens.
    ///
    /// If the chain has never seen the `prev` tokens together, `None` is returned.
    pub fn generate_next_token(
        &self,
        rng: &mut impl Rng,
        prev: &TokenPairRef<'_>,
    ) -> Option<TokenRef<'_>> {
        let dist = self.map.get(prev)?;
        Some(dist.get_random_token(rng))
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
        prev: &TokenPairRef<'_>,
        n: usize,
    ) -> Option<Vec<TokenRef<'_>>> {
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
        prev: &TokenPairRef<'_>,
        n: usize,
    ) -> Option<Vec<TokenRef<'_>>> {
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

/// The result of feeding some tokens to a [`ChainBuilder`]. The `Err` variant means that the feed
/// failed, and that an unmodified [`ChainBuilder`] was returned.
///
/// Can be converted to a [`ChainBuilder`] using [`IntoChainBuilder::into_cb()`].
///
/// # Examples
///
/// ```
/// # use markovish::{ChainBuilder, chain::FeedResult};
/// use markovish::IntoChainBuilder;
///
/// let mut cb: ChainBuilder = ChainBuilder::new();
/// let feed_result: FeedResult = cb.feed_str("I am fed.");
/// cb = feed_result.into_cb();
/// ```
pub type FeedResult = Result<UpdatedChainBuilder, ChainBuilder>;

/// Builds a Chain by being fed strings and keeping track of the likelihood that one token
/// follows two others.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    /// Will return an error if the builder have not been fed any strings.
    pub fn build(self) -> Result<Chain, ChainBuilder> {
        if self.map.is_empty() {
            return Err(self);
        }

        let mut chain_map = HashMap::with_capacity(self.map.len());
        for (pair, dist_builder) in self.map {
            chain_map.insert(pair, dist_builder.build());
        }

        Ok(Chain { map: chain_map })
    }

    /// Add the occurance of `next` following `prev`.
    pub fn add_occurance(&mut self, prev: &TokenPairRef<'_>, next: &str) -> AddedPair {
        match self.map.get_mut(&prev) {
            Some(b) => {
                b.add_token(next);
                AddedPair::Updated
            }
            None => {
                let mut b = TokenDistributionBuilder::new();
                b.add_token(next);
                let tp = TokenPair::from(prev);
                self.map.insert(tp, b);
                AddedPair::New
            }
        }
    }

    /// Feeds the chain builder with more text, adding the tokens in this string to the mappings of
    /// this. May fail if the input string is too short.
    ///
    /// The tokens are from [`unicode_segmentation::UnicodeSegmentation::split_word_bounds()`]; if
    /// you want more control you can pre-split your tokens and use
    /// [`ChainBuilder::feed_tokens()`], but using a builder fed with both strings and pre-split
    /// tokens might result in odd output.
    ///
    /// See also [`ChainBuilder::feed_tokens()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use markovish::ChainBuilder;
    /// use markovish::IntoChainBuilder;
    ///
    /// let mut cb = ChainBuilder::new();
    ///
    /// // Chaining calls are easy, since the result can be used as a [`ChainBuilder`] using
    /// // the `IntoChainBuilder::into_cb` method
    /// cb = cb.feed_str("") // Won't feed, since we don't have enough tokens
    ///         .into_cb() // We ignore if we succeeded
    ///         .feed_str("Hello Tokens!") // Ok!
    ///         .into_cb()
    ///         .feed_str("I ") // Too few tokens again...
    ///         .into_cb();
    /// ```
    pub fn feed_str(self, content: &str) -> FeedResult {
        let tokens = content.split_word_bounds();
        self.feed_tokens(tokens)
    }

    /// Feeds the chain builder with pre-split tokens. Useful if you want to just split on
    /// whitespace and then join the result. May fail if the input is too short, in which case
    /// the (not updated) [`ChainBuilder`] is returned.
    ///
    /// If used *together* with [`ChainBuilder::feed_str()`], the result may be odd, since
    /// the different sets of token pairs may not collide enough.
    pub fn feed_tokens<'a, T: Iterator<Item = TokenRef<'a>>>(mut self, tokens: T) -> FeedResult {
        let mut windows = tokens.tuple_windows();
        let mut new_pairs = 0_usize;
        let mut updated_pairs = 0_usize;

        // We should add at least one
        if let Some((left, right, next)) = windows.next() {
            match self.add_occurance(&(left, right), next) {
                AddedPair::New => new_pairs += 1,
                AddedPair::Updated => updated_pairs += 1,
            }
        } else {
            return Err(self);
        }

        for (left, right, next) in windows {
            match self.add_occurance(&(left, right), next) {
                AddedPair::New => new_pairs += 1,
                AddedPair::Updated => updated_pairs += 1,
            }
        }

        Ok(UpdatedChainBuilder {
            chain_builder: self,
            new_pairs,
            updated_pairs,
        })
    }
}

impl Default for ChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The result of feeding tokens to a [`ChainBuilder`], where tokens were
/// added. Contains data about what was updated.
///
/// This is a thin wrapper around a [`ChainBuilder`].
///
/// # Examples
///
/// ```
/// use markovish::{ChainBuilder, IntoChainBuilder, chain::UpdatedChainBuilder};
///
/// let updated: UpdatedChainBuilder = ChainBuilder::new().feed_str("Hello there").unwrap();
/// println!("Added {} new token pairs and updated {}", updated.new_pairs, updated.updated_pairs);
/// let cb: ChainBuilder = updated.into();
/// ```
#[derive(Debug)]
pub struct UpdatedChainBuilder {
    /// The wrapped updated [`ChainBuilder`]
    pub chain_builder: ChainBuilder,
    /// The amount of [`TokenPair`]s that were seen for the first time in
    /// this update.
    pub new_pairs: usize,
    /// The amount of times existing [`TokenPair`]s had their distribution updated.
    pub updated_pairs: usize,
}

impl From<UpdatedChainBuilder> for ChainBuilder {
    fn from(value: UpdatedChainBuilder) -> Self {
        value.chain_builder
    }
}

impl From<FeedResult> for ChainBuilder {
    fn from(value: FeedResult) -> Self {
        match value {
            Ok(ucb) => ucb.chain_builder,
            Err(cb) => cb,
        }
    }
}

/// Marker result for [`ChainBuilder::add_occurance()`] to indicate if a [`TokenPair`] had been
/// seen before or not.
///
/// Does not contain information about if the next token had been seen before or not.
pub enum AddedPair {
    /// This pair was new.
    New,
    /// This pair existed and the matching next token has been incremented.
    Updated,
}

/// We're sealing [`IntoChainBuilder`] by using a supertrait. We want other crates to be
/// able to call `into_cb`, but not to implement it themselves. So this trait should *never* be public.
///
/// See `<https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed>`.
///
/// # Examples
///
/// ```fail_compile
/// use markovish::chain::SealedIntoChainBuilder;
///
/// struct MyStruct();
///
/// impl SealedIntoChainBuilder for MyStruct {}
/// ```
trait SealedIntoChainBuilder {}
impl SealedIntoChainBuilder for FeedResult {}
impl SealedIntoChainBuilder for UpdatedChainBuilder {}

/// Sealed trait used to make a type convertable to a [`ChainBuilder`].
///
/// You cannot implement this by yourself, but you can use its method
/// (or well, you could fork the whole crate I guess...).
#[allow(private_bounds)]
pub trait IntoChainBuilder: SealedIntoChainBuilder {
    /// Returns the inner [`ChainBuilder`].
    fn into_cb(self) -> ChainBuilder;
}

impl IntoChainBuilder for FeedResult {
    fn into_cb(self) -> ChainBuilder {
        match self {
            Ok(ucb) => ucb.chain_builder,
            Err(cb) => cb,
        }
    }
}

impl IntoChainBuilder for UpdatedChainBuilder {
    fn into_cb(self) -> ChainBuilder {
        self.chain_builder
    }
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;

    use crate::{chain::IntoChainBuilder, distribution::TokenDistribution, Chain, ChainBuilder};

    #[test]
    #[should_panic]
    fn empty_chain_builder_panics() {
        let _ = Chain::builder().build().unwrap();
    }

    #[test]
    #[should_panic]
    fn empty_token_dist_builder_panics() {
        let _ = TokenDistribution::builder().build();
    }

    #[test]
    fn feed_too_few_tokens() {
        // Only 2, we need three
        let s = "I ";
        assert!(Chain::builder().feed_str(s).is_err());
    }

    #[test]
    fn simple_single_possible_token() {
        let s = "I am";
        let cb = Chain::builder().feed_str(s).into_cb();
        let chain = cb.build().unwrap();
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
        let cb = Chain::builder().feed_str(s).into_cb();
        let chain = cb.build().unwrap();
        assert!(chain
            .generate_next_token(&mut thread_rng(), &("You", " "))
            .is_none());
    }

    #[test]
    fn simple_generate_max_n_tokens() {
        let s = "I am-full!of?cats";
        let cb = Chain::builder().feed_str(s).into_cb();
        let chain = cb.build().unwrap();

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
        let cb = Chain::builder().feed_str(s).into_cb();
        let chain = cb.build().unwrap();
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
        let cb = Chain::builder().feed_str(s).into_cb();
        let chain = cb.build().unwrap();
        assert!(chain
            .generate_max_n_tokens(&mut thread_rng(), &("I", " "), 0)
            .unwrap()
            .is_empty())
    }

    #[test]
    fn simple_generate_max_n_tokens_impossible_first() {
        let s = "I am-full!of?cats";
        let cb = Chain::builder().feed_str(s).into_cb();
        let chain = cb.build().unwrap();
        assert!(chain
            .generate_max_n_tokens(&mut thread_rng(), &("You", " "), 13)
            .is_none())
    }

    #[test]
    fn simple_generate_n_tokens_zero() {
        let s = "I am-full!of?cats";
        let cb = Chain::builder().feed_str(s).into_cb();
        let chain = cb.build().unwrap();
        assert!(chain
            .generate_n_tokens(&mut thread_rng(), &("I", " "), 0)
            .unwrap()
            .is_empty())
    }

    #[test]
    fn simple_generate_n_tokens_impossible_first() {
        let s = "I am-full!of?cats";
        let cb = Chain::builder().feed_str(s).into_cb();
        let chain = cb.build().unwrap();
        assert!(chain
            .generate_n_tokens(&mut thread_rng(), &("You", " "), 13)
            .is_none())
    }

    #[test]
    fn generate_long_from_start_tokens() {
        // Nice output from fortune
        let s = r#"
Coach: How's it going, Norm?
Norm:  Daddy's rich and Momma's good lookin'.
                -- Cheers, Truce or Consequences

Sam:   What's up, Norm?
Norm:  My nipples.  It's freezing out there.
                -- Cheers, Coach Returns to Action

Coach: What's the story, Norm?
Norm:  Thirsty guy walks into a bar.  You finish it.
                -- Cheers, Endless Slumper
"#;
        let cb = Chain::builder().feed_str(s).into_cb();
        let chain = cb.build().unwrap();
        let mut rng = thread_rng();
        for _ in 0..100 {
            let start = chain.start_tokens(&mut rng).unwrap();
            let _ = chain.generate_n_tokens(&mut rng, &start.as_ref(), 100);
        }
    }

    #[test]
    fn generate_long_using_generate_str() {
        let s = r#"
The difference between a program and a script isn't as subtle as most people
think. A script is interpreted, and a program is compiled.

Of course, there's no reason you can't write a compiler that immediately
executes the compiled form of a program without writing compilation artifacts
to disk, but that's an implementation detail, and precision in technical
matters is important.

Though Perl 5, for example, doesn't write out the artifacts of compilation to
disk and Java and .Net do, Perl 5 is clearly an interpreter even though it
evaluates the compiled form of code in the same way that the JVM and the CLR
do. Why? Because it's a scripting language.

Okay, that's a facetious explanation.

The difference between a program and a script is if there's native compilation
available in at least one widely-used implementation. Thus Java before the
prevalence of even the HotSpot JVM and its JIT was a scripting language and
now it's a programming language, except that you can write a C interpreter
that doesn't have a JIT and C programs become scripts.

    -- chromatic
    -- "Program vs. Script" ( http://use.perl.org/~chromatic/journal/35804 )
        "#;

        let chain = Chain::from_text(s).unwrap();
        for _ in 0..100 {
            chain.generate_str(&mut thread_rng(), 100).unwrap();
        }
    }

    #[test]
    fn get_pairs() {
        let s = r#"
This is a text.
There are many like it, but this one is mine.
        -- Unknown
        "#;
        let chain = Chain::from_text(s).unwrap();
        let good_starting_points: Vec<_> =
            chain.pairs().filter(|tp| tp.0.as_str() == "\n").collect();
        assert_eq!(good_starting_points.len(), 3);
    }

    #[test]
    fn feed_stats() {
        let cb = ChainBuilder::new();

        // `end` is never in a TokenPair, it's just added to ("hi", "hi")
        let ucb = cb
            .feed_tokens("hi hi what hi hi end".split_whitespace())
            .unwrap();

        assert_eq!(ucb.new_pairs, 3);
        assert_eq!(ucb.updated_pairs, 1, "hi hi should be updated once");
    }
}
