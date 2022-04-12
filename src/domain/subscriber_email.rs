use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid email", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests_email {

    use super::*;
    use claim::assert_err;
    use fake::faker::internet::fr_fr::SafeEmail;
    use fake::Fake;
    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(_: &mut quickcheck::Gen) -> Self {
            let mut ng = rand::thread_rng();
            let email = SafeEmail().fake_with_rng(&mut ng);
            dbg!(&email);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_email_are_parsed_succesfuly(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberEmail::parse(name));
    }

    #[test]
    fn email_missing_symbol_is_rejected() {
        let name = "bla.com".to_string();
        assert_err!(SubscriberEmail::parse(name));
    }

    #[test]
    fn emai_missin_sibject_is_rejected() {
        let name = "@bla.com".to_string();
        assert_err!(SubscriberEmail::parse(name));
    }
}
