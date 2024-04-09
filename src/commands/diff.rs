use crate::*;

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Opts {
    /// First file to compare, represented in output as < (blue)
    file_one: PathBuf,
    /// Second file to compare, represented in output as > (red)
    file_two: PathBuf,
    /// Show all lines, not just the ones that differ
    #[clap(short = 'a', long, default_value = "false")]
    show_all: bool,
    /// Hide line numbers
    #[clap(short = 'n', long, default_value = "false")]
    hide_line_number: bool,
    /// Number of lines around a change to print (only used if -a is not set)
    #[clap(short, long, default_value = "2")]
    context: usize,
}

pub async fn main(opts: &Opts) -> Result<()> {
    let file_one = std::fs::read_to_string(&opts.file_one)
        .map_err(|e| anyhow!("Unable to read file '{:?}': {}", &opts.file_one, e))?
        .lines()
        .map(|l| l.to_string())
        .collect::<Vec<_>>();
    let file_two = std::fs::read_to_string(&opts.file_two)
        .map_err(|e| anyhow!("Unable to read file '{:?}': {}", &opts.file_two, e))?
        .lines()
        .map(|l| l.to_string())
        .collect::<Vec<_>>();

    let max_lines = std::cmp::max(file_one.len(), file_two.len());
    let results = (0..max_lines)
        .map(|line_idx| DiffDirection::new(file_one.get(line_idx), file_two.get(line_idx)))
        .collect::<Vec<_>>();

    let mut differing_idxs = results
        .iter()
        .enumerate()
        .filter(|(_, result)| !result.is_equal())
        .map(|(idx, _)| idx);

    let mut last_printed_idx = None;
    let display_skipped_idx_count =
        |current_idx: usize, last_printed_idx: Option<usize>| -> Option<String> {
            match current_idx
                .saturating_sub(1)
                .checked_sub(last_printed_idx.unwrap_or_default())
            {
                None | Some(0) => None,
                Some(1) => Some("...\t1 line".into()),
                Some(skipped_count) => Some(format!("...\t{skipped_count} lines")),
            }
            .map(|s| s.italic().dimmed().to_string())
        };

    let mut next_differing_idx = differing_idxs.next();
    for (idx, result) in results.iter().enumerate() {
        if let Some(differing_idx) = next_differing_idx {
            if idx > differing_idx + opts.context {
                next_differing_idx = differing_idxs.next();
            }
        }
        if !opts.show_all {
            if next_differing_idx.is_none() {
                // We've reached the end of the differing lines
                break;
            }
            if result.is_equal() {
                if let Some(differing_idx) = next_differing_idx {
                    if idx < differing_idx.saturating_sub(opts.context) {
                        continue;
                    }
                }
            }
        }

        if let Some(skipped_lines) = display_skipped_idx_count(idx, last_printed_idx) {
            println!("{}", skipped_lines);
        }

        last_printed_idx = Some(idx);

        result.get_output().iter().for_each(|v| {
            if opts.hide_line_number {
                println!("{v}")
            } else {
                println!("{line_num: >4} {v}", line_num = idx + 1)
            }
        })
    }

    if last_printed_idx.is_none() {
        println!("Files are identical");
    } else if let Some(skipped_lines) =
        display_skipped_idx_count(max_lines.saturating_sub(1), last_printed_idx)
    {
        println!("{}", skipped_lines);
    }

    Ok(())
}

/// Represents the direction of a difference between two inputs, represented as 'left' and 'right'
enum DiffDirection<'a> {
    Left(&'a String),
    Right(&'a String),
    Both(&'a String, &'a String),
    Equal(&'a String),
}

impl DiffDirection<'_> {
    fn new<'a>(v1: Option<&'a String>, v2: Option<&'a String>) -> DiffDirection<'a> {
        match (v1, v2) {
            (Some(v1), None) => DiffDirection::Left(v1),
            (None, Some(v2)) => DiffDirection::Right(v2),
            (Some(v1), Some(v2)) => {
                if v1 != v2 {
                    DiffDirection::Both(v1, v2)
                } else {
                    DiffDirection::Equal(v1)
                }
            }
            (None, None) => unreachable!(),
        }
    }
    fn get_output(&self) -> Vec<String> {
        match self {
            DiffDirection::Left(v) => vec![format!("{}| {}", "<".blue(), v)],
            DiffDirection::Right(v) => vec![format!("{}| {}", ">".red(), v)],
            DiffDirection::Both(v1, v2) => vec![
                format!("{}| {}", "<".blue(), v1),
                format!("{}| {}", ">".red(), v2),
            ],
            DiffDirection::Equal(v) => vec![format!("{}| {}", "=".green(), v)],
        }
    }
    fn is_equal(&self) -> bool {
        matches!(self, DiffDirection::Equal(_))
    }
}
