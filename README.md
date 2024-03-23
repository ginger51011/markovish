<div align="center">
    <h1>⛓️markovish💬</h1>
    <i>Dead simple Markov chain text generation</i>
<br />
<br />

[![Static Badge](https://img.shields.io/badge/GitHub-ginger51011%2Fmarkovish-FFA400?style=flat&logo=github)](https://github.com/ginger51011/markovish)
[![Crates.io (markovish)](https://img.shields.io/crates/v/markovish)](https://crates.io/crates/markovish)
[![GitHub License](https://img.shields.io/github/license/ginger51011/markovish)](https://github.com/ginger51011/markovish/blob/main/LICENSE)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ginger51011/markovish/rust.yml)](https://github.com/ginger51011/markovish/actions/)
[![docs.rs](https://img.shields.io/docsrs/markovish)](https://docs.rs/markovish/latest/markovish/)
</div>

Super simple (and fast) Markov chains in Rust.

```rust
use markovish::Chain;

let fortune = r#"
This is a test of the Emergency Broadcast System.  If this had been an
actual emergency, do you really think we'd stick around to tell you?
"#;

let chain = Chain::from_text(fortune).unwrap();
let new_fortune = chain.generate_str(&mut rand::thread_rng(), 300);
```

This project is mostly for my personal use in [`pandoras_pot`](https://github.com/ginger51011/pandoras_pot/), but PRs
and issues are welcome.

This crate is aimed to be very simple, and the current aim is to do the following well:

1. Take some input text.
2. Parse it.
3. Generate an infinite string of new text using that.

More information about usage and the like can be found in the [crate docs](https://docs.rs/markovish/latest/markovish/)
and in the [examples](./examples).

If you want to save a chain, you can enable the `serde` feature and serialize it.

# Support

I do not accept any donations. If you however find any software I
write for fun useful, please consider donating to an efficient charity that
save or improve lives the most per `$CURRENCY`.

[GiveWell.org](https://givewell.org) is an excellent website that can help you
donate to the world's most efficient charities. Alternatives listing the current
best charities for helping our planet is [Founders Pledge](https://www.founderspledge.com/funds/climate-change-fund), and for
animal welfare [Animal Charity Evaluators](https://animalcharityevaluators.org/donation-advice/recommended-charity-fund/).

- Residents of Sweden can do tax-deductable donations to GiveWell via [Ge Effektivt](https://geeffektivt.se)
- Residents of Norway can do the same via [Gi Effektivt](https://gieffektivt.no/)

This list is not exhaustive; your country may have an equivalent.
