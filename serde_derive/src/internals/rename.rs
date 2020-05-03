//! Code to convert the Rust-styled field/variant (e.g. `my_field`, `MyType`) to the
//! case of the source (e.g. `my-field`, `MY_FIELD`).

// See https://users.rust-lang.org/t/psa-dealing-with-warning-unused-import-std-ascii-asciiext-in-today-s-nightly/13726
#[allow(deprecated, unused_imports)]
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::str::FromStr;

use self::ConvertCase::*;

/// The different possible ways to change case of fields in a struct, or variants in an enum.
#[derive(Copy, Clone, PartialEq)]
pub enum ConvertCase {
    /// Rename direct children to "lowercase" style.
    LowerCase,
    /// Rename direct children to "UPPERCASE" style.
    UPPERCASE,
    /// Rename direct children to "PascalCase" style, as typically used for
    /// enum variants.
    PascalCase,
    /// Rename direct children to "camelCase" style.
    CamelCase,
    /// Rename direct children to "snake_case" style, as commonly used for
    /// fields.
    SnakeCase,
    /// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
    /// used for constants.
    ScreamingSnakeCase,
    /// Rename direct children to "kebab-case" style.
    KebabCase,
    /// Rename direct children to "SCREAMING-KEBAB-CASE" style.
    ScreamingKebabCase,
}

/// Add a prefix to fields in a struct, or variants in an enum.
#[derive(Clone, PartialEq)]
pub struct AddPrefix(String);

/// The container-level rename rules to apply to a field
#[derive(Clone, PartialEq, Default)]
pub struct RenameRule {
    prefix: Option<AddPrefix>,
    convert_case: Option<ConvertCase>,
}

impl RenameRule {
    pub fn new(prefix: Option<AddPrefix>, convert_case: Option<ConvertCase>) -> Self {
        RenameRule {
            prefix,
            convert_case,
        }
    }

    /// Apply a renaming rule to an enum variant, returning the version expected in the source.
    pub fn apply_to_variant(&self, name: &str) -> String {
        let mut name = Cow::from(name);
        if let Some(convert_case) = self.convert_case {
            name = Cow::from(convert_case.apply_to_variant(&name))
        }
        if let Some(prefix) = &self.prefix {
            name = Cow::from(prefix.apply(&name))
        }
        name.into_owned()
    }

    /// Apply a renaming rule to a struct field, returning the version expected in the source.
    pub fn apply_to_field(&self, name: &str) -> String {
        let mut name = Cow::from(name);
        if let Some(convert_case) = self.convert_case {
            name = Cow::from(convert_case.apply_to_field(&name))
        }
        if let Some(prefix) = &self.prefix {
            name = Cow::from(prefix.apply(&name))
        }
        name.into_owned()
    }
}

impl ConvertCase {
    /// Apply a renaming rule to an enum variant, returning the version expected in the source.
    pub fn apply_to_variant(&self, variant: &str) -> String {
        match *self {
            PascalCase => variant.to_owned(),
            LowerCase => variant.to_ascii_lowercase(),
            UPPERCASE => variant.to_ascii_uppercase(),
            CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
            SnakeCase => {
                let mut snake = String::new();
                for (i, ch) in variant.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake.push('_');
                    }
                    snake.push(ch.to_ascii_lowercase());
                }
                snake
            }
            ScreamingSnakeCase => SnakeCase.apply_to_variant(variant).to_ascii_uppercase(),
            KebabCase => SnakeCase.apply_to_variant(variant).replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase
                .apply_to_variant(variant)
                .replace('_', "-"),
        }
    }

    /// Apply a renaming rule to a struct field, returning the version expected in the source.
    pub fn apply_to_field(&self, field: &str) -> String {
        match *self {
            LowerCase | SnakeCase => field.to_owned(),
            UPPERCASE => field.to_ascii_uppercase(),
            PascalCase => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            CamelCase => {
                let pascal = PascalCase.apply_to_field(field);
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            ScreamingSnakeCase => field.to_ascii_uppercase(),
            KebabCase => field.replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase.apply_to_field(field).replace('_', "-"),
        }
    }
}

impl AddPrefix {
    pub fn from_string(prefix: String) -> AddPrefix {
        AddPrefix(prefix)
    }

    pub fn apply(&self, name: &str) -> String {
        let mut result = String::with_capacity(self.0.len() + name.len());
        result.push_str(&self.0);
        result.push_str(&name);
        result
    }
}

impl FromStr for ConvertCase {
    type Err = ();

    fn from_str(rename_all_str: &str) -> Result<Self, Self::Err> {
        match rename_all_str {
            "lowercase" => Ok(LowerCase),
            "UPPERCASE" => Ok(UPPERCASE),
            "PascalCase" => Ok(PascalCase),
            "camelCase" => Ok(CamelCase),
            "snake_case" => Ok(SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(ScreamingSnakeCase),
            "kebab-case" => Ok(KebabCase),
            "SCREAMING-KEBAB-CASE" => Ok(ScreamingKebabCase),
            _ => Err(()),
        }
    }
}

#[test]
fn rename_variants() {
    for &(original, lower, upper, camel, snake, screaming, kebab, screaming_kebab) in &[
        (
            "Outcome", "outcome", "OUTCOME", "outcome", "outcome", "OUTCOME", "outcome", "OUTCOME",
        ),
        (
            "VeryTasty",
            "verytasty",
            "VERYTASTY",
            "veryTasty",
            "very_tasty",
            "VERY_TASTY",
            "very-tasty",
            "VERY-TASTY",
        ),
        ("A", "a", "A", "a", "a", "A", "a", "A"),
        ("Z42", "z42", "Z42", "z42", "z42", "Z42", "z42", "Z42"),
    ] {
        assert_eq!(LowerCase.apply_to_variant(original), lower);
        assert_eq!(UPPERCASE.apply_to_variant(original), upper);
        assert_eq!(PascalCase.apply_to_variant(original), original);
        assert_eq!(CamelCase.apply_to_variant(original), camel);
        assert_eq!(SnakeCase.apply_to_variant(original), snake);
        assert_eq!(ScreamingSnakeCase.apply_to_variant(original), screaming);
        assert_eq!(KebabCase.apply_to_variant(original), kebab);
        assert_eq!(
            ScreamingKebabCase.apply_to_variant(original),
            screaming_kebab
        );
    }
}

#[test]
fn rename_fields() {
    for &(original, upper, pascal, camel, screaming, kebab, screaming_kebab) in &[
        (
            "outcome", "OUTCOME", "Outcome", "outcome", "OUTCOME", "outcome", "OUTCOME",
        ),
        (
            "very_tasty",
            "VERY_TASTY",
            "VeryTasty",
            "veryTasty",
            "VERY_TASTY",
            "very-tasty",
            "VERY-TASTY",
        ),
        ("a", "A", "A", "a", "A", "a", "A"),
        ("z42", "Z42", "Z42", "z42", "Z42", "z42", "Z42"),
    ] {
        assert_eq!(UPPERCASE.apply_to_field(original), upper);
        assert_eq!(PascalCase.apply_to_field(original), pascal);
        assert_eq!(CamelCase.apply_to_field(original), camel);
        assert_eq!(SnakeCase.apply_to_field(original), original);
        assert_eq!(ScreamingSnakeCase.apply_to_field(original), screaming);
        assert_eq!(KebabCase.apply_to_field(original), kebab);
        assert_eq!(ScreamingKebabCase.apply_to_field(original), screaming_kebab);
    }
}
