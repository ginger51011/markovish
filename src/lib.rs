//! Dead simple text generation using markov chains. The text generator behind [`pandoras_pot`](https://github.com/ginger51011/pandoras_pot/).
//!
//! Right now this generator only supports second order Markov chains, that is, it looks at two
//! symbols at a time and then guesses what the third might be (weighted depending on how likely
//! the three-word combination is in the source text). The randomness is built using a weighted
//! distribution (see [`rand_distr::weighted_alias::WeightedAliasIndex`]).
//!
//! `markovish` uses [`hashbrown`](https://crates.io/crates/hashbrown) internally for extra speed.
//! However, the default hasher used by `hashbrown` does not provide the same level of protection
//! against HashDoS attacks as the standard library hasher. If you are only going to use `markovish`
//! on texts you trust, you can ignore this.
//!
//! ```
//! use markovish::Chain;
//! let fortune = r#"
//! This is a test of the Emergency Broadcast System.  If this had been an
//! actual emergency, do you really think we'd stick around to tell you?
//! "#;
//! let mut cb = Chain::builder();
//! cb.feed_str(fortune);
//! let chain = cb.build();
//!
//! // This can be any pair of tokens in `fortune`
//! let mut rngod = rand::thread_rng();
//! let start = chain.start_tokens(&mut rngod).unwrap();
//! let new_fortune = chain.generate_n_tokens(&mut rngod, &start.as_ref(), 30);
//! ```
//!
//! # Features
//!
//! `markovish` comes with some feature(s) that you can disable (or enable) at will. They are:
//!
//! - `inline-more`: Enables the [`hashbrown`](https://crates.io/crates/hashbrown) `inline-more`
//! feature, improving performance at the cost of compilation time. Enabled by default.

pub mod chain;
pub mod distribution;
pub mod token;

pub use chain::{Chain, ChainBuilder};
