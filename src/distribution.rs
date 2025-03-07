//! [`TokenDistribution`] are representations of how common [`Token`]s are, and are paired up with
//! a [`TokenPair`](crate::token::TokenPair) in a [`Chain`](crate::Chain).

use hashbrown::HashMap;
use rand::Rng;
use rand_distr::{Distribution, weighted::WeightedAliasIndex};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::token::Token;

/// A distribution of choices and their likelyhood.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TokenDistribution {
    /// Mappings of index in choices to their likelyhood.
    dist: WeightedAliasIndex<u64>,
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TokenDistributionBuilder {
    /// Counts how many times a token is likely to appear.
    map: HashMap<String, u64>,
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
