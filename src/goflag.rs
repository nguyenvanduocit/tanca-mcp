//! A small command-line flag parser that follows Go's `flag` package:
//! `-name value`, `-name=value`, `--name value` and `--name=value` forms,
//! parsing stops at the first non-flag argument or after `--`, `-h`/`-help`
//! prints usage and exits 0, and errors print a message plus usage and exit 2.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    String,
    Int,
}

#[derive(Debug, Clone)]
struct Flag {
    kind: Kind,
    usage: String,
    default: String,
}

/// A set of string and integer flags.
#[derive(Debug, Clone)]
pub struct FlagSet {
    name: String,
    flags: BTreeMap<String, Flag>,
}

/// Why [`FlagSet::parse`] stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// `-h` or `-help` was given.
    Help,
    /// Invalid input; the message matches Go's.
    Invalid(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Help => f.write_str("flag: help requested"),
            ParseError::Invalid(m) => f.write_str(m),
        }
    }
}

/// Parsed flag values.
#[derive(Debug, Clone)]
pub struct Values {
    values: BTreeMap<String, String>,
    ints: BTreeMap<String, i64>,
    /// Arguments left after flag parsing.
    pub args: Vec<String>,
}

impl Values {
    /// Value of a string flag.
    pub fn str(&self, name: &str) -> String {
        self.values
            .get(name)
            .unwrap_or_else(|| panic!("undefined string flag {name}"))
            .clone()
    }

    /// Value of an int flag.
    pub fn int(&self, name: &str) -> i64 {
        *self
            .ints
            .get(name)
            .unwrap_or_else(|| panic!("undefined int flag {name}"))
    }
}

impl FlagSet {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            flags: BTreeMap::new(),
        }
    }

    /// Defines a string flag.
    pub fn string(&mut self, name: &str, default: &str, usage: &str) -> &mut Self {
        self.define(name, Kind::String, default.to_string(), usage)
    }

    /// Defines an int flag.
    pub fn int(&mut self, name: &str, default: i64, usage: &str) -> &mut Self {
        self.define(name, Kind::Int, default.to_string(), usage)
    }

    fn define(&mut self, name: &str, kind: Kind, default: String, usage: &str) -> &mut Self {
        self.flags.insert(
            name.to_string(),
            Flag {
                kind,
                usage: usage.to_string(),
                default,
            },
        );
        self
    }

    /// Parses `args` (without the program or command name).
    pub fn parse<S: AsRef<str>>(&self, args: &[S]) -> Result<Values, ParseError> {
        let mut values = BTreeMap::new();
        let mut ints = BTreeMap::new();
        for (name, f) in &self.flags {
            values.insert(name.clone(), f.default.clone());
            if f.kind == Kind::Int {
                ints.insert(name.clone(), f.default.parse().unwrap_or(0));
            }
        }

        let args: Vec<String> = args.iter().map(|a| a.as_ref().to_string()).collect();
        let mut i = 0;
        while i < args.len() {
            let s = &args[i];
            if s.len() < 2 || !s.starts_with('-') {
                break;
            }
            let mut num_minuses = 1;
            if s.as_bytes()[1] == b'-' {
                num_minuses += 1;
                if s.len() == 2 {
                    // "--" terminates the flags.
                    i += 1;
                    break;
                }
            }
            let rest = &s[num_minuses..];
            if rest.is_empty() || rest.starts_with('-') || rest.starts_with('=') {
                return Err(ParseError::Invalid(format!("bad flag syntax: {s}")));
            }
            i += 1;

            let (name, inline_value) = match rest.split_once('=') {
                Some((n, v)) => (n, Some(v.to_string())),
                None => (rest, None),
            };

            let Some(flag) = self.flags.get(name) else {
                if name == "help" || name == "h" {
                    return Err(ParseError::Help);
                }
                return Err(ParseError::Invalid(format!(
                    "flag provided but not defined: -{name}"
                )));
            };

            let value = match inline_value {
                Some(v) => v,
                None if i < args.len() => {
                    i += 1;
                    args[i - 1].clone()
                }
                None => {
                    return Err(ParseError::Invalid(format!(
                        "flag needs an argument: -{name}"
                    )));
                }
            };

            if flag.kind == Kind::Int {
                let n = parse_go_int(&value).map_err(|e| {
                    ParseError::Invalid(format!("invalid value {value:?} for flag -{name}: {e}"))
                })?;
                ints.insert(name.to_string(), n);
            }
            values.insert(name.to_string(), value);
        }

        Ok(Values {
            values,
            ints,
            args: args[i..].to_vec(),
        })
    }

    /// Parses `args`; on error prints the message and usage to stderr and
    /// exits (0 for help, 2 otherwise), like `flag.ExitOnError`.
    pub fn parse_or_exit<S: AsRef<str>>(&self, args: &[S]) -> Values {
        match self.parse(args) {
            Ok(v) => v,
            Err(ParseError::Help) => {
                eprint!("{}", self.usage());
                std::process::exit(0);
            }
            Err(ParseError::Invalid(msg)) => {
                eprintln!("{msg}");
                eprint!("{}", self.usage());
                std::process::exit(2);
            }
        }
    }

    /// Usage text in the format of Go's `flag.PrintDefaults`.
    pub fn usage(&self) -> String {
        let mut out = format!("Usage of {}:\n", self.name);
        for (name, f) in &self.flags {
            let type_name = match f.kind {
                Kind::String => "string",
                Kind::Int => "int",
            };
            let mut line = format!("  -{name} {type_name}");
            line.push_str("\n    \t");
            line.push_str(&f.usage.replace('\n', "\n    \t"));
            let is_zero = match f.kind {
                Kind::String => f.default.is_empty(),
                Kind::Int => f.default == "0",
            };
            if !is_zero {
                match f.kind {
                    Kind::String => line.push_str(&format!(" (default {:?})", f.default)),
                    Kind::Int => line.push_str(&format!(" (default {})", f.default)),
                }
            }
            out.push_str(&line);
            out.push('\n');
        }
        out
    }
}

/// Parses an integer like Go's `strconv.ParseInt(s, 0, 64)`: optional sign,
/// `0x`/`0o`/`0b`/leading-`0` prefixes and `_` separators after a prefix.
fn parse_go_int(s: &str) -> Result<i64, &'static str> {
    let (neg, body) = match s.as_bytes().first() {
        Some(b'+') => (false, &s[1..]),
        Some(b'-') => (true, &s[1..]),
        _ => (false, s),
    };
    if body.is_empty() {
        return Err("parse error");
    }
    let lower = body.to_ascii_lowercase();
    let (radix, digits, prefixed) = if let Some(d) = lower.strip_prefix("0x") {
        (16, d.to_string(), true)
    } else if let Some(d) = lower.strip_prefix("0o") {
        (8, d.to_string(), true)
    } else if let Some(d) = lower.strip_prefix("0b") {
        (2, d.to_string(), true)
    } else if lower.len() > 1 && lower.starts_with('0') {
        (8, lower[1..].to_string(), true)
    } else {
        (10, lower.clone(), false)
    };
    if digits.contains('_') {
        // Underscores are only allowed between digits after a base prefix.
        let ok = prefixed
            && !digits.starts_with('_')
            && !digits.ends_with('_')
            && !digits.contains("__");
        if !ok {
            return Err("parse error");
        }
    }
    let digits = digits.replace('_', "");
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return Err("parse error");
    }
    let magnitude = u64::from_str_radix(&digits, radix).map_err(|_| "value out of range")?;
    if neg {
        if magnitude > i64::MAX as u64 + 1 {
            return Err("value out of range");
        }
        Ok((magnitude as i128).wrapping_neg() as i64)
    } else {
        i64::try_from(magnitude).map_err(|_| "value out of range")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set() -> FlagSet {
        let mut fs = FlagSet::new("list-requests");
        fs.string("env", "", "Path to .env file")
            .string("output", "text", "Output format: text or json")
            .int("page", 1, "Page number")
            .int(
                "status",
                -1,
                "Status filter: 0=pending, 1=approved, 2=rejected. -1 for all.",
            );
        fs
    }

    #[test]
    fn parses_all_forms() {
        let v = set()
            .parse(&[
                "-env",
                "a",
                "--output=json",
                "--page",
                "3",
                "-status=-1",
                "rest",
            ])
            .unwrap();
        assert_eq!(v.str("env"), "a");
        assert_eq!(v.str("output"), "json");
        assert_eq!(v.int("page"), 3);
        assert_eq!(v.int("status"), -1);
        assert_eq!(v.args, vec!["rest"]);
    }

    #[test]
    fn defaults_apply() {
        let v = set().parse::<&str>(&[]).unwrap();
        assert_eq!(v.str("output"), "text");
        assert_eq!(v.int("page"), 1);
        assert_eq!(v.str("env"), "");
    }

    #[test]
    fn errors_match_go() {
        let fs = set();
        assert_eq!(fs.parse(&["-h"]).unwrap_err(), ParseError::Help);
        assert_eq!(
            fs.parse(&["--bogus"]).unwrap_err(),
            ParseError::Invalid("flag provided but not defined: -bogus".into())
        );
        assert_eq!(
            fs.parse(&["--page", "abc"]).unwrap_err(),
            ParseError::Invalid("invalid value \"abc\" for flag -page: parse error".into())
        );
        assert_eq!(
            fs.parse(&["--page"]).unwrap_err(),
            ParseError::Invalid("flag needs an argument: -page".into())
        );
        assert_eq!(
            fs.parse(&["---x"]).unwrap_err(),
            ParseError::Invalid("bad flag syntax: ---x".into())
        );
    }

    #[test]
    fn usage_matches_go() {
        assert_eq!(
            set().usage(),
            "Usage of list-requests:\n  -env string\n    \tPath to .env file\n  -output string\n    \tOutput format: text or json (default \"text\")\n  -page int\n    \tPage number (default 1)\n  -status int\n    \tStatus filter: 0=pending, 1=approved, 2=rejected. -1 for all. (default -1)\n"
        );
    }

    #[test]
    fn go_int_syntax() {
        assert_eq!(parse_go_int("0x10"), Ok(16));
        assert_eq!(parse_go_int("010"), Ok(8));
        assert_eq!(parse_go_int("-0b1_1"), Ok(-3));
        assert_eq!(parse_go_int("1_0"), Err("parse error"));
        assert_eq!(
            parse_go_int("99999999999999999999"),
            Err("value out of range")
        );
        assert_eq!(parse_go_int("-9223372036854775808"), Ok(i64::MIN));
    }
}
