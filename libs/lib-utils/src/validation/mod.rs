pub mod macros;

use std::fmt::Debug;
use regex::Regex;

// traits

pub trait Rule: std::fmt::Display + std::fmt::Debug { }

pub trait Validate<V>: Rule {
    fn validate(&self, value: &V) -> Result<&Self>;
}

pub fn validate_rules<'a,T,U>(value: &T, rules: &[&'a U]) -> Box<[Error<'a>]>
where
    U: Validate<T>
{
    let mut errors: Vec<Error> = Vec::new();

    for &rule in rules {
        let res = rule.validate(value);
        if let Err(e) = res {
            errors.push(e);
        }
    }

    errors.into_boxed_slice()
}

// Standart rules

#[derive(Debug, Clone)]
pub enum Rules {
    MaxLength(i16),
    MinLength(i16),
    ContainsDidgits(bool),
    ContainsSpecialCharacters(bool),
    ContainsLowecaseCharacter(bool),
    ContainsUppercaseCharacter(bool),
}

impl Rule for Rules { }

impl std::fmt::Display for Rules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rules::MaxLength(max_length) => { write!(f, "Maximum length must be: {}", max_length) },
            Rules::MinLength(min_length) => { write!(f, "Minimum length must be: {}", min_length) },
            Rules::ContainsDidgits(must_contain) => {
                if *must_contain {
                    write!(f, "Must contain at least one digit")
                } else {
                    write!(f, "Must not contain digits")
                }
            },
            Rules::ContainsSpecialCharacters(must_contain) => {
                if *must_contain {
                    write!(f, "Must contain at least one special character")
                } else {
                    write!(f, "Must not contain special characters")
                }
            },
            Rules::ContainsLowecaseCharacter(must_contain) => {
                if *must_contain {
                    write!(f, "Must contain at least one lowercase character")
                } else {
                    write!(f, "Must not contain lowecase characters")
                }
            },
            Rules::ContainsUppercaseCharacter(must_contain) => {
                if *must_contain {
                    write!(f, "Must contain at least one uppercase character")
                } else {
                    write!(f, "Must not contain uppercase characters")
                }
            },
        }
    }
}

impl Validate<String> for Rules where {
    fn validate(&self, value: &String) -> Result<&Self> {
        match self {
            Rules::MaxLength(length) => {
                let re = format!("^.{{0,{}}}$",length);
                let regex = Regex::new(&re).unwrap();

                if !regex.is_match(value) {
                    Err(Error::RuleNotValidated(self))
                } else {
                    Ok(self)
                }
            },
            Rules::MinLength(length) => {
                let re = format!("^.{{{},}}$",length);
                let regex = Regex::new(&re).unwrap();

                if !regex.is_match(value) {
                    Err(Error::RuleNotValidated(self))
                } else {
                    Ok(self)
                }
            },
            Rules::ContainsDidgits(must_contain) => {
                let re = r"\d";
                let regex = Regex::new(re).unwrap();
                let do_contain = regex.is_match(value);

                if *must_contain {
                    if do_contain {
                        Ok(self)
                    } else {
                        Err(Error::RuleNotValidated(self))
                    }
                } else if do_contain {
                    Err(Error::RuleNotValidated(self))
                } else {
                    Ok(self)
                }

            },
            Rules::ContainsSpecialCharacters(must_contain) => {
                let re = r".*[@$!%*?&]";
                let regex = Regex::new(re).unwrap();
                let do_contain = regex.is_match(value);

                if *must_contain {
                    if do_contain {
                        Ok(self)
                    } else {
                        Err(Error::RuleNotValidated(self))
                    }
                } else if do_contain {
                    Err(Error::RuleNotValidated(self))
                } else {
                    Ok(self)
                }
            },
            Rules::ContainsLowecaseCharacter(must_contain) => {
                let re = r".*[a-z]";
                let regex = Regex::new(re).unwrap();
                let do_contain = regex.is_match(value);

                if *must_contain {
                    if do_contain {
                        Ok(self)
                    } else {
                        Err(Error::RuleNotValidated(self))
                    }
                } else if do_contain {
                    Err(Error::RuleNotValidated(self))
                } else {
                    Ok(self)
                }
            },
            Rules::ContainsUppercaseCharacter(must_contain) => {
                let re = r".*[A-Z]";
                let regex = Regex::new(re).unwrap();
                let do_contain = regex.is_match(value);

                if *must_contain {
                    if do_contain {
                        Ok(self)
                    } else {
                        Err(Error::RuleNotValidated(self))
                    }
                } else if do_contain {
                    Err(Error::RuleNotValidated(self))
                } else {
                    Ok(self)
                }
            },
        }
    }
}

// Error

pub type Result<'a,T> = std::result::Result<T,Error<'a>>;

#[derive(Debug, Clone)]
pub enum Error<'a> {
    RuleNotValidated(&'a dyn Rule)
}

impl<'a> std::fmt::Display for Error<'a> {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter
    ) -> core::result::Result<(),core::fmt::Error> {
        match self {
            Error::RuleNotValidated(rule) => write!(f,"{}",rule)
        }
    }
}

impl<'a> serde::Serialize for Error<'a>  {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.serialize_str(&format!("{}",self))
    }
}

impl<'a> std::error::Error for Error<'a> {}
