use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, bail, Context};
use num::{CheckedAdd, Integer};

/// A number input can be a single number, a range, or a step range
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NumberRange<T>(pub Vec<T>)
where
    T: Integer + FromStr + Copy + Display + CheckedAdd;

impl<T: Integer + FromStr + Copy + Display + CheckedAdd> Display for NumberRange<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, v) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{v}")?;
        }
        Ok(())
    }
}
impl<T: Integer + FromStr<Err = TErr> + Copy + Display + CheckedAdd, TErr: Display>
    std::str::FromStr for NumberRange<T>
{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            bail!("Empty input");
        }
        if let Ok(n) = s.parse::<T>() {
            return Ok(NumberRange(vec![n]));
        } else if s.contains("..") {
            let mut parts = s.split("..");
            let start = parts
                .next()
                .context("missing start value")?
                .parse::<T>()
                .map_err(|e| anyhow!("invalid start value: {e}"))?;
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
            return Ok(NumberRange(values));
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
            return Ok(NumberRange(values));
        }
        bail!("Invalid input: {s}")
    }
}
