use std::ops::Range;

use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

pub const TOKEN_MATCHER_PATTERN: &str = "\\$(-?\\d+)(:)?(-?\\d+)?"; //"\\$\\d+:?(?:\\d+)?";

lazy_static! {
    static ref TOKEN_MATCHER_REGEX: Regex = Regex::new(TOKEN_MATCHER_PATTERN).unwrap();
}

pub enum ArgumentToken {
    Single(i64),
    Joint(i64, Option<i64>),
}

pub struct Argument {
    pub range: Range<usize>,
    pub token: ArgumentToken,
}

pub struct Expression<'a> {
    args: Vec<Argument>,
    text: &'a str,
}

impl<'a> Expression<'a> {
    pub fn parse(text: &'a str) -> Expression {
        Expression {
            text,
            args: TOKEN_MATCHER_REGEX
                .captures_iter(text)
                .map(|capture| {
                    let mat = capture.get(0).unwrap();
                    // let expr = text.get(mat.range()).unwrap();

                    let group1 = capture.get(1).unwrap();
                    let is_range = capture.get(2).is_some();
                    let group2 = capture.get(3);

                    let begin: i64 = group1.as_str().parse().unwrap();
                    let token = match is_range {
                        true => match group2 {
                            Some(group2_unwrapped) => ArgumentToken::Joint(
                                begin,
                                Some(group2_unwrapped.as_str().parse::<i64>().unwrap()),
                            ),
                            None => ArgumentToken::Joint(begin, None),
                        },
                        false => ArgumentToken::Single(begin),
                    };
                    Argument {
                        range: mat.range(),
                        token,
                    }
                })
                .collect(),
        }
    }

    pub fn replace(&self, str_arguments: &[&str]) -> Result<String, String> {
        let mut copy = self.text.to_string();
        for arg in self.args.iter().rev() {
            Self::replace_arg(arg, &mut copy, str_arguments)?;
        }

        Ok(copy)
    }

    fn clamp(i: i64, len: usize) -> Result<usize, String> {
        // -1 -> len - 1
        if i < 0 {
            return Ok((len as u64 - i.unsigned_abs()) as usize);
        }

        if i.unsigned_abs() >= len as u64 {
            return Err(format!("No argument found at index {i}"));
        }

        Ok(i as usize)
    }

    fn replace_arg(arg: &Argument, text: &mut String, arguments: &[&str]) -> Result<(), String> {
        let len = arguments.len();

        match arg.token {
            ArgumentToken::Single(start) => {
                let replacement = arguments.get(Self::clamp(start, len)?).unwrap();

                text.replace_range(arg.range.clone(), replacement);
            }
            ArgumentToken::Joint(start, e) => {
                if start >= len as i64 {
                    return Err(format!(
                        "No joint argument found at index start {start}, length is {len}",
                    ));
                }
                let clamped_start = Self::clamp(start, len)?;

                // left to right iter
                let mut lfr_skipped_iter = arguments.iter().skip(clamped_start);

                let replacement = match e {
                    Some(end) => {
                        if end > len as i64 {
                            return Err(format!(
                                "No joint argument found at index end {end}, length is {len}",
                            ));
                        }

                        let clamped_end = Self::clamp(end, len)?;

                        // reverse order
                        // + 1 to include itself

                        let diff = clamped_end.abs_diff(clamped_start) + 1;
                        if clamped_start > clamped_end {
                            // right to left iter
                            arguments
                                .iter()
                                .skip(clamped_end)
                                .rev()
                                .take(diff)
                                .join(" ")
                        } else {
                            lfr_skipped_iter.take(diff).join(" ")
                        }
                    }
                    None => lfr_skipped_iter.join(" "),
                };

                text.replace_range(arg.range.clone(), &replacement);
            }
        };

        Ok(())
    }
}
