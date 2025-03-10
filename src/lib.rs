#![forbid(unsafe_code)]
//! Dead simple text generation using markov chains. The text generator behind [`pandoras_pot`](https://github.com/ginger51011/pandoras_pot/).
//!
//! Right now this generator only supports second order Markov chains, that is, it looks at two
//! tokens at a time and then guesses what the third might be (weighted depending on how likely
//! the three-token combination is in the source text). The randomness is built using a weighted
//! distribution (see [`rand_distr::weighted_alias::WeightedAliasIndex`]). See [`token`] for more
//! information about what defines a token.
//!
//! `markovish` uses [`hashbrown`](https://crates.io/crates/hashbrown) internally for extra speed.
//! However, the default hasher used by `hashbrown` does not provide the same level of protection
//! against HashDoS attacks as the standard library hasher. If you are only going to use `markovish`
//! on texts you trust, you can ignore this.
//!
//! ```
//! use markovish::Chain;
//!
//! let fortune = r#"
//! This is a test of the Emergency Broadcast System.  If this had been an
//! actual emergency, do you really think we'd stick around to tell you?
//! "#;
//! let chain = Chain::from_text(fortune).unwrap();
//! let new_fortune = chain.generate_str(&mut rand::thread_rng(), 300);
//! ```
//!
//! The examples in this crate use [`rand::thread_rng()`], but if you want things to go fast you
//! could try using [`rand::rngs::SmallRng`](https://docs.rs/rand/latest/rand/rngs/struct.SmallRng.html),
//! which is generally faster but not as safe (but you should NOT use this crate to generate passwords
//! anyway).
//!
//! # Features
//!
//! `markovish` comes with some feature(s) that you can disable (or enable) at will. They are:
//!
//! - `inline-more`: Enables the [`hashbrown`](https://crates.io/crates/hashbrown) `inline-more`
//!   feature, improving performance at the cost of compilation time. Enabled by default.
//! - `serde`: Allows for serializing and deserializing some of the data structures in this library,
//!   so they can be stored and reused once created. Especially serializing [`Chain`] and [`ChainBuilder`]
//!   is useful, since the same chain can be recreated without having to parse the text again.

pub mod chain;
pub mod distribution;
pub mod token;

pub use chain::{Chain, ChainBuilder, IntoChainBuilder};
