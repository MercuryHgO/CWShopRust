pub mod validation;

#[cfg(test)]
pub mod tests {
    extern crate proc_macro;

    use std::error::Error;

    use crate::{validate, validation::{self, validate_rules, Rule, Rules, Validate}};

    #[test]
    fn rules_validation_many_test() {
        let f = "Zhopa".to_string();
        let result = validate_rules(&f, &[
            &Rules::MaxLength(2),
            &Rules::MinLength(10)
        ]);

        result.iter().for_each(
            |e| {
                eprintln!("{:?}",e);
            }
        );
    }

    #[test]
    fn borrowing() {
        let str = "Zhopa".to_string();
        Rules::MaxLength(100).validate(&str);
        println!("{}",str);
    }

    #[test]
    fn rules_validation_test() -> Result<(),Box<dyn Error>>
    {
        macro_rules! assert_rule {
            ( $must_validate:expr, $must_not_validate:expr ) => {
                let _ = $must_validate.inspect_err( |v| { panic!("Must be validated: {v}") } );
                let _ = $must_not_validate.inspect( |v| { panic!("Must not be validated: {v}") } );
            };
        }

        assert_rule!(
            Rules::MaxLength(10).validate(&"Aboba".to_string()),
            Rules::MaxLength(3).validate(&"Aboba".to_string())
        );
        

        assert_rule!(
            Rules::MinLength(4).validate(&"Aboba".to_string()),
            Rules::MinLength(10).validate(&"Aboba".to_string())
        );

        assert_rule!(
            Rules::ContainsDidgits(true).validate(&"Apanki123".to_string()),
            Rules::ContainsDidgits(true).validate(&"Apanki".to_string())
        );

        assert_rule!(
            Rules::ContainsDidgits(false).validate(&"Apanki".to_string()),
            Rules::ContainsDidgits(false).validate(&"Apanki123".to_string())
        );

        assert_rule!(
            Rules::ContainsSpecialCharacters(true).validate(&"Apanki@".to_string()),
            Rules::ContainsSpecialCharacters(true).validate(&"Apanki".to_string())
        );

        assert_rule!(
            Rules::ContainsSpecialCharacters(false).validate(&"Apanki".to_string()),
            Rules::ContainsSpecialCharacters(false).validate(&"Apanki@".to_string())
        );

        assert_rule!(
            Rules::ContainsLowecaseCharacter(true).validate(&"Apanki@".to_string()),
            Rules::ContainsLowecaseCharacter(true).validate(&"APANKI".to_string())
        );

        assert_rule!(
            Rules::ContainsLowecaseCharacter(false).validate(&"APANKI".to_string()),
            Rules::ContainsLowecaseCharacter(false).validate(&"Apanki@".to_string())
        );

        assert_rule!(
            Rules::ContainsUppercaseCharacter(true).validate(&"Apanki@".to_string()),
            Rules::ContainsUppercaseCharacter(true).validate(&"apanki".to_string())
        );

        assert_rule!(
            Rules::ContainsUppercaseCharacter(false).validate(&"apanki@".to_string()),
            Rules::ContainsUppercaseCharacter(false).validate(&"APANKI".to_string())
        );

        Ok(())
    }
}
