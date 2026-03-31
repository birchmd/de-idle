use std::collections::VecDeque;

pub mod exponential;
pub mod linear;
pub mod quadratic;
pub mod sinusoidal;

#[cfg(test)]
mod tests;

pub type GoalCheckerFn = fn(&VecDeque<(f64, f64)>) -> bool;
