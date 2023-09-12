use super::Parser;
use crate::parsers::{CommandArg, CommandArgParseError, ParseInput};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Time {
    Ticks(f32),
    Seconds(f32),
    Days(f32),
}

impl CommandArg for Time {
    fn parse_arg(input: &mut ParseInput) -> Result<Self, CommandArgParseError> {
        input.skip_whitespace();
        let mut number_str = String::new();
        while let Some(c) = input.pop() {
            match c {
                't' => {
                    return Ok(Time::Ticks(number_str.parse::<f32>().map_err(|_| {
                        CommandArgParseError::InvalidArgument(
                            "time".to_string(),
                            "not a valid time".to_string(),
                        )
                    })?));
                }
                's' => {
                    return Ok(Time::Seconds(number_str.parse::<f32>().map_err(|_| {
                        CommandArgParseError::InvalidArgument(
                            "time".to_string(),
                            "not a valid time".to_string(),
                        )
                    })?));
                }
                'd' => {
                    return Ok(Time::Days(number_str.parse::<f32>().map_err(|_| {
                        CommandArgParseError::InvalidArgument(
                            "time".to_string(),
                            "not a valid time".to_string(),
                        )
                    })?));
                }
                _ => {
                    number_str.push(c);
                }
            }
        }
        if !number_str.is_empty() {
            return Ok(Time::Ticks(number_str.parse::<f32>().map_err(|_| {
                CommandArgParseError::InvalidArgument(
                    "time".to_string(),
                    "not a valid time".to_string(),
                )
            })?));
        }

        Err(CommandArgParseError::InvalidArgument(
            "time".to_string(),
            "not a valid time".to_string(),
        ))
    }

    fn display() -> Parser {
        Parser::Time
    }
}

#[test]
fn test_time() {
    let mut input = ParseInput::new("42.31t".to_string());
    let time = Time::parse_arg(&mut input).unwrap();
    assert_eq!(time, Time::Ticks(42.31));

    let mut input = ParseInput::new("42.31".to_string());
    let time = Time::parse_arg(&mut input).unwrap();
    assert_eq!(time, Time::Ticks(42.31));

    let mut input = ParseInput::new("1239.72s".to_string());
    let time = Time::parse_arg(&mut input).unwrap();
    assert_eq!(time, Time::Seconds(1239.72));

    let mut input = ParseInput::new("133.1d".to_string());
    let time = Time::parse_arg(&mut input).unwrap();
    assert_eq!(time, Time::Days(133.1));
}
