use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);
impl SubscriberName {
    pub fn parse(s: &str) -> Result<SubscriberName, String> {
        let is_empty = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_characters = ['<', '>', '[', ']', '(', ')', '"', '\'', '/', '\\'];
        let has_invalid_chars = forbidden_characters.iter().any(|x| s.contains(*x));

        if is_empty || is_too_long || has_invalid_chars {
            Err(format!(
                "`{}` est une valeure d'entrée inccorect pour le nom",
                s
            ))
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_is_valid() {
        let name = "a".repeat(256);
        assert_ok!(SubscriberName::parse(&name));
    }

    #[test]
    fn a_258_graphem_long_is_not_valid() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(&name));
    }

    #[test]
    fn white_space_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(&name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(&name));
    }

    #[test]
    fn invalid_char_are_rejected() {
        for name in ["<", "'", "\"", "/"] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(&name));
        }
    }

    #[test]
    fn un_nom_valide_est_parse_crrectement() {
        let name = "Ursula le Guin".to_string();
        assert_ok!(SubscriberName::parse(&name));
    }
}
