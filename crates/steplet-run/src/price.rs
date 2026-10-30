//! What Steplet's own agent spends (the Nebius plan, Task 2). Money is counted in billionths of a dollar, in whole
//! numbers, so no rounding creeps into the spending cap. Prices are Nebius Token Factory's, per token; the real
//! calls of Task 4 check them against what Nebius charged.

use std::iter::Sum;
use std::ops::{Add, AddAssign};

use crate::model::Usage;

/// An amount of money, in billionths of a dollar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nanodollars(pub u64);

impl Nanodollars {
    pub const fn from_cents(cents: u64) -> Nanodollars {
        Nanodollars(cents * 10_000_000)
    }

    pub fn dollars(self) -> f64 {
        self.0 as f64 / 1e9
    }
}

impl Add for Nanodollars {
    type Output = Nanodollars;
    fn add(self, other: Nanodollars) -> Nanodollars {
        Nanodollars(self.0.saturating_add(other.0))
    }
}

impl AddAssign for Nanodollars {
    fn add_assign(&mut self, other: Nanodollars) {
        *self = *self + other;
    }
}

impl Sum for Nanodollars {
    fn sum<I: Iterator<Item = Nanodollars>>(iter: I) -> Nanodollars {
        iter.fold(Nanodollars(0), Add::add)
    }
}

/// What one token costs, sent and received.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Price {
    pub input: Nanodollars,
    pub output: Nanodollars,
}

/// Nebius Token Factory's prices for the Nemotron models Steplet knows, per token (a price per million tokens of
/// $0.30 is 300 billionths of a dollar per token). Model names are compared without regard to case.
const PRICES: &[(&str, u64, u64)] = &[
    ("nvidia/nemotron-3-super-120b-a12b", 300, 900),
    ("nvidia/Nemotron-3-Ultra-550b-a55b", 1000, 3000),
    ("nvidia/NVIDIA-Nemotron-3-Nano-30B-A3B", 60, 240),
];

/// The names the screens give the models Steplet knows. Model names are compared without regard to case.
const NAMES: &[(&str, &str)] = &[
    ("nvidia/nemotron-3-super-120b-a12b", "Nemotron 3 Super"),
    ("nvidia/Nemotron-3-Ultra-550b-a55b", "Nemotron 3 Ultra"),
    ("nvidia/NVIDIA-Nemotron-3-Nano-30B-A3B", "Nemotron 3 Nano"),
];

/// The model's name on screen ("Nemotron 3 Super"), or its id as Steplet asked for it when Steplet has no name for it.
pub fn model_name(model: &str) -> String {
    NAMES.iter().find(|(id, _)| id.eq_ignore_ascii_case(model)).map_or(model, |&(_, name)| name).to_string()
}

/// The model's price, or `None` for a model Steplet has no price for: such a model is never run, because its spending
/// could not be capped.
pub fn price(model: &str) -> Option<Price> {
    PRICES
        .iter()
        .find(|(name, _, _)| name.eq_ignore_ascii_case(model))
        .map(|&(_, input, output)| Price { input: Nanodollars(input), output: Nanodollars(output) })
}

/// What one answer cost.
pub fn cost(u: &Usage, p: &Price) -> Nanodollars {
    Nanodollars(u.prompt_tokens.saturating_mul(p.input.0)) + Nanodollars(u.completion_tokens.saturating_mul(p.output.0))
}

/// The most one call can cost: every byte sent counted as a token (a token is never shorter than one byte), plus
/// 1000 tokens for what the service adds around the messages, plus the longest answer allowed.
pub fn worst_case(body_bytes: usize, max_tokens: u32, p: &Price) -> Nanodollars {
    let sent = (body_bytes as u64).saturating_add(1000);
    cost(&Usage { prompt_tokens: sent, completion_tokens: u64::from(max_tokens) }, p)
}
