use std::cmp::Ordering;
use std::ops::{Add, Mul, Sub};

pub mod event;
pub mod trigger;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CriteriaOperator {
    #[default]
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

impl CriteriaOperator {
    #[must_use]
    pub fn is_valid<T: PartialOrd>(&self, current: &T, target: &T) -> bool {
        match self {
            CriteriaOperator::Equal => current.partial_cmp(target) == Some(Ordering::Equal),
            CriteriaOperator::NotEqual => current.partial_cmp(target) != Some(Ordering::Equal),
            CriteriaOperator::LessThan => current < target,
            CriteriaOperator::LessOrEqual => current <= target,
            CriteriaOperator::GreaterThan => current > target,
            CriteriaOperator::GreaterOrEqual => current >= target,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ModifierOperator {
    #[default]
    Set,
    Add,
    Subtract,
    Multiply,
}

impl ModifierOperator {
    #[must_use]
    pub fn apply<T>(&self, current: T, amount: T) -> T
    where
        T: Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
    {
        match self {
            ModifierOperator::Set => amount,
            ModifierOperator::Add => current + amount,
            ModifierOperator::Subtract => current - amount,
            ModifierOperator::Multiply => current * amount,
        }
    }
}
