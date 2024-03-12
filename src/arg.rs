use std::{fmt::format, ops::Range};

use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

pub const TOKEN_MATCHER_PATTERN: &str = r"\$(-?\d+)(:)?(-?\d+)?(\?)?";

lazy_static! {
    static ref TOKEN_MATCHER_REGEX: Regex = Regex::new(TOKEN_MATCHER_PATTERN).unwrap();
}

pub enum ArgumentToken {
    Single(i64),
    Joint(i64, Option<i64>),
}

pub struct Argument {
    pub range: Range<usize>,
    pub optional: bool,
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
                    // the whole argument
                    let mat = capture.get(0).unwrap();

                    let group1 = capture.get(1).unwrap(); // first number
                    let is_range = capture.get(2).is_some(); // :
                    let group2 = capture.get(3); // second number
                    let optional = capture.get(4).is_some(); // is optional

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
                        optional,
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

    fn clamp_and_reverse(i: i64, len: usize) -> Result<usize, String> {
        // -1 -> len - 1
        if i < 0 {
            return Ok((len as u64 - i.unsigned_abs()) as usize);
        }

        Ok(i as usize)
    }

    fn replace_arg(arg: &Argument, text: &mut String, arguments: &[&str]) -> Result<(), String> {
        let len = arguments.len();

        let mut replace_empty = || {
            text.replace_range(arg.range.clone(), "");
        };

        match arg.token {
            ArgumentToken::Single(start) => {
                let replacement = arguments.get(Self::clamp_and_reverse(start, len)?).cloned();

                // Just return
                if arg.optional && replacement.is_none() {
                    replace_empty();
                    return Ok(());
                }

                text.replace_range(arg.range.clone(), replacement.unwrap());
            }
            ArgumentToken::Joint(start, e) => {
                if start >= len as i64 {
                    return Err(format!(
                        "No joint argument found at index start {start}, length is {len}",
                    ));
                }
                let clamped_start = Self::clamp_and_reverse(start, len)?;

                // left to right iter
                let mut lfr_skipped_iter = arguments.iter().skip(clamped_start);

                match lfr_skipped_iter.len() {
                    0 if arg.optional => {
                        replace_empty();
                        return Ok(());
                    }
                    0 => return Err(format!("Start index {start} is greater than length {len}")),
                    _ => (),
                };

                let replacement = match e {
                    // end is defined
                    Some(end) => {
                        // elements less than end
                        if end >= len as i64 && !arg.optional {
                            return Err(format!(
                                "No joint argument found at index end {end}, length is {len}",
                            ));
                        }

                        let clamped_end = Self::clamp_and_reverse(end, len)?;

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
                    // until end of args
                    None => lfr_skipped_iter.join(" "),
                };

                text.replace_range(arg.range.clone(), &replacement);
            }
        };

        Ok(())
    }
}
