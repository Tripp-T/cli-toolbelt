use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, bail};
use num::{CheckedAdd, Integer};

/// A number input can be a single number, a range, or a step range
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NumberInput<T>
where
    T: Integer + FromStr + Copy + Display + CheckedAdd,
{
    pub input: String,
    pub values: Vec<T>,
}

impl<T: Integer + FromStr + Copy + Display + CheckedAdd> Display for NumberInput<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values = self
            .values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{}", values)
    }
}

impl<T: Integer + FromStr + Copy + Display + CheckedAdd> std::str::FromStr for NumberInput<T> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(anyhow::anyhow!("Empty input"));
        }
        if let Ok(n) = s.parse::<T>() {
            return Ok(NumberInput {
                input: s.to_string(),
                values: vec![n],
            });
        } else if s.contains("..") {
            let mut parts = s.split("..");
            let start = parts
                .next()
                .ok_or_else(|| anyhow::anyhow!("No start value"))?
                .parse::<T>()
                .map_err(|_| anyhow!("invalid start value"))?;
            let mut inclusive_end = false;
            let end = parts
                .next()
                .map(|s| {
                    if s.starts_with('=') {
                        inclusive_end = true;
                        s.trim_start_matches('=')
                    } else {
                        s
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("No end value"))?
                .parse::<T>()
                .map_err(|_| anyhow!("invalid end value"))?;
            let step = parts
                .next()
                .map(|s| s.parse::<T>())
                .transpose()
                .map_err(|_| anyhow::anyhow!("Invalid step value"))?
                .unwrap_or_else(|| {
                    if start < end {
                        T::one()
                    } else {
                        T::zero().sub(T::one())
                    }
                });
            let mut values = Vec::new();
            if inclusive_end {
                for i in num::range_step_inclusive(start, end, step) {
                    values.push(i);
                }
            } else {
                for i in num::range_step(start, end, step) {
                    values.push(i);
                }
            }
            return Ok(NumberInput {
                input: s.to_string(),
                values,
            });
        } else if s.contains('-') {
            let mut parts = s.split('-');
            let start = parts
                .next()
                .ok_or_else(|| anyhow::anyhow!("No start value"))?
                .parse::<T>()
                .map_err(|_| anyhow!("invalid start"))?;
            let end = parts
                .next()
                .map(|s| {
                    if s.starts_with('=') {
                        s.trim_start_matches('=')
                    } else {
                        s
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("No end value"))?
                .parse::<T>()
                .map_err(|_| anyhow!("invalid end value"))?;
            let step = parts
                .next()
                .map(|s| s.parse::<T>())
                .transpose()
                .map_err(|_| anyhow::anyhow!("Invalid step value: "))?
                .unwrap_or_else(|| {
                    if start < end {
                        T::one()
                    } else {
                        T::zero().sub(T::one())
                    }
                });
            let mut values = Vec::new();
            for i in num::range_step_inclusive(start, end, step) {
                values.push(i);
            }
            return Ok(NumberInput {
                input: s.to_string(),
                values,
            });
        }
        bail!("Invalid input: {s}")
    }
}
