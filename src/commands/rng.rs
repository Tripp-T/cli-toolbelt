use anyhow::{anyhow, Result};
use clap::Parser;
use rand::Rng;

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Opts {
    /// range to generate between, inclusive of beginning and end  (e.g. 15-20, 20)
    /// or number of dice to roll (e.g. 2d20)
    #[clap(required = true)]
    ranges: Vec<RngRange>,
    /// quiet mode
    /// if sum is specified, only the sum will be printed
    #[clap(short, long)]
    quiet: bool,
    /// sum the results
    #[clap(short, long)]
    sum: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct RngRange {
    bottom: u128,
    top: u128,
    count: u128,
}

impl std::str::FromStr for RngRange {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut count = 1;
        let mut bottom = 1;
        let top: u128;

        let parse_err = || anyhow!("failed to parse '{s}' as u128");

        if s.contains('-') {
            let parts = s.split('-').collect::<Vec<_>>();
            if parts.len() != 2 {
                return Err(anyhow!("invalid range: {}", s));
            }
            bottom = parts[0].parse::<u128>().map_err(|_| parse_err())?;
            top = parts[1].parse::<u128>().map_err(|_| parse_err())?;
        } else if s.contains('d') {
            let parts = s.split('d').collect::<Vec<_>>();
            if parts.len() != 2 {
                return Err(anyhow!("invalid range: {}", s));
            }
            count = parts[0].parse::<u128>().map_err(|_| parse_err())?;
            top = parts[1].parse::<u128>().map_err(|_| parse_err())?;
        } else {
            top = s.parse::<u128>().map_err(|_| parse_err())?;
        }

        if bottom > top {
            return Err(anyhow!(
                "bottom of range must be less than or equal to top of range"
            ));
        }

        Ok(RngRange { bottom, top, count })
    }
}

pub async fn main(opts: &Opts) -> Result<()> {
    let mut rng = rand::thread_rng();
    let mut sum: u128 = 0;
    for input_range in &opts.ranges {
        for _ in 1..=input_range.count {
            let roll = rng.gen_range(input_range.bottom..=input_range.top);
            sum += roll;
            if opts.quiet {
                if opts.sum {
                    continue;
                } else {
                    println!("{}", roll);
                    continue;
                }
            }
            println!("{}-{}:\t{}", input_range.bottom, input_range.top, roll);
        }
    }
    if opts.sum {
        if opts.quiet {
            println!("{}", sum);
        } else {
            println!("sum:\t{}", sum);
        }
    }
    Ok(())
}
