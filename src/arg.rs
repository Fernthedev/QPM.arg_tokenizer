use std::ops::Range;

use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

pub const TOKEN_MATCHER_PATTERN: &str = "\\$(\\d+)(:)?(\\d+)?"; //"\\$\\d+:?(?:\\d+)?";

lazy_static! {
    static ref TOKEN_MATCHER_REGEX: Regex = Regex::new(TOKEN_MATCHER_PATTERN).unwrap();
}

pub enum ArgumentToken {
    Single(usize),
    Joint(usize, Option<usize>),
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

                    let begin: usize = group1.as_str().parse().unwrap();
                    let token = match is_range {
                        true => match group2 {
                            Some(group2_unwrapped) => ArgumentToken::Joint(
                                begin,
                                Some(group2_unwrapped.as_str().parse::<usize>().unwrap()),
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

    fn replace_arg(arg: &Argument, text: &mut String, arguments: &[&str]) -> Result<(), String> {
        match arg.token {
            ArgumentToken::Single(i) => {
                let replacement = arguments
                    .get(i)
                    .ok_or_else(|| format!("No argument found at index {i}"))?;

                text.replace_range(arg.range.clone(), replacement);
            }
            ArgumentToken::Joint(start, e) => {
                if start >= arguments.len() {
                    return Err(format!(
                        "No joint argument found at index start {start}, length is {}",
                        arguments.len()
                    ));
                }

                // left to right iter
                let mut lfr_skipped_iter = arguments.iter().skip(start);

                let replacement = match e {
                    Some(end) => {
                        if end > arguments.len() {
                            return Err(format!(
                                "No joint argument found at index end {end}, length is {}",
                                arguments.len()
                            ));
                        }

                        // reverse order
                        // + 1 to include itself

                        let diff = end.abs_diff(start) + 1;
                        if start > end {
                            // right to left iter
                            arguments.iter().skip(end).rev().take(diff).join(" ")
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
