//! Validating instances parsed by `jiter`. jiter borrows strings from the input text and keeps objects as
//! vectors of pairs, so parsing an instance allocates far less than building a `serde_json::Value`.

use std::borrow::Cow;

use jiter::JsonValue;
use jsonschema::{
    JsonType,
    json::{
        Array,
        Json,
        JsonNumber,
        Node,
        NodeIdentity,
        Object,
    },
};
use serde_json::{
    Map,
    Number,
    Value,
};

/// The jiter representation of a JSON document, for `jsonschema::options_for::<Jiter>()`.
pub(crate) struct Jiter;

/// Parse `text` into a jiter value. `NaN` and `Infinity` are rejected, as JSON does.
///
/// # Errors
///
/// Returns an error if `text` is not JSON, or nests deeper than jiter's limit of 200 levels.
pub(crate) fn parse(text: &str) -> Result<JsonValue<'_>, jiter::JsonError> {
    JsonValue::parse(text.as_bytes(), false)
}

impl Json for Jiter {
    type Node<'a> = &'a JsonValue<'a>;
    type PreparedKey = String;
    // A string node can borrow the name it holds, so there is nothing to reuse between calls.
    type StringBuffer = ();

    // An object is a vector of pairs, so a lookup scans it. One pass over its members is never slower than
    // two lookups.
    const KEYS_PER_LOOKUP: usize = 2;

    fn prepare_key(key: &str) -> String {
        key.to_owned()
    }

    fn with_string_node<T>(
        _buffer: &mut (),
        string: &str,
        f: impl FnOnce(Self::Node<'_>) -> T,
    ) -> T {
        f(&JsonValue::Str(Cow::Borrowed(string)))
    }
}

impl<'a> Node<'a, Jiter> for &'a JsonValue<'a> {
    type Object = &'a [(Cow<'a, str>, JsonValue<'a>)];
    type Array = &'a [JsonValue<'a>];
    type Number = JiterNumber<'a>;

    fn as_object(&self) -> Option<Self::Object> {
        match self {
            JsonValue::Object(members) => Some(members.as_slice()),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<Self::Array> {
        match self {
            JsonValue::Array(items) => Some(items.as_slice()),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<Cow<'a, str>> {
        match self {
            JsonValue::Str(string) => Some(Cow::Borrowed(string.as_ref())),
            _ => None,
        }
    }

    fn as_number(&self) -> Option<JiterNumber<'a>> {
        match self {
            JsonValue::Int(_) | JsonValue::BigInt(_) | JsonValue::Float(_) => {
                Some(JiterNumber(self))
            }
            _ => None,
        }
    }

    fn as_boolean(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(boolean) => Some(*boolean),
            _ => None,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    fn json_type(&self) -> JsonType {
        match self {
            JsonValue::Null => JsonType::Null,
            JsonValue::Bool(_) => JsonType::Boolean,
            JsonValue::Int(_) | JsonValue::BigInt(_) | JsonValue::Float(_) => JsonType::Number,
            JsonValue::Str(_) => JsonType::String,
            JsonValue::Array(_) => JsonType::Array,
            JsonValue::Object(_) => JsonType::Object,
        }
    }

    fn to_value(&self) -> Cow<'a, Value> {
        Cow::Owned(to_serde_value(self))
    }

    fn identity(&self) -> Option<NodeIdentity> {
        Some(NodeIdentity::new(
            std::ptr::from_ref::<JsonValue<'a>>(self) as usize
        ))
    }
}

impl<'a> Object<'a, Jiter> for &'a [(Cow<'a, str>, JsonValue<'a>)] {
    type Node = &'a JsonValue<'a>;
    type MemberName = &'a str;
    type MembersIter = JiterMembersIter<'a>;

    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn get(&self, key: &String) -> Option<&'a JsonValue<'a>> {
        // jiter keeps every occurrence of a duplicated key. Searching from the end finds the last one, the
        // occurrence `serde_json` would have kept.
        self.iter()
            .rev()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    fn members(&self) -> JiterMembersIter<'a> {
        JiterMembersIter(self.iter())
    }
}

/// The members of a jiter object, in document order.
pub(crate) struct JiterMembersIter<'a>(std::slice::Iter<'a, (Cow<'a, str>, JsonValue<'a>)>);

impl<'a> Iterator for JiterMembersIter<'a> {
    type Item = (&'a str, &'a JsonValue<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(name, value)| (name.as_ref(), value))
    }
}

impl<'a> Array<'a, Jiter> for &'a [JsonValue<'a>] {
    type Node = &'a JsonValue<'a>;
    type ElementsIter = std::slice::Iter<'a, JsonValue<'a>>;

    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn elements(&self) -> std::slice::Iter<'a, JsonValue<'a>> {
        self.iter()
    }
}

/// A jiter number: always an `Int`, `BigInt` or `Float` value.
pub(crate) struct JiterNumber<'a>(&'a JsonValue<'a>);

impl JsonNumber for JiterNumber<'_> {
    fn as_u64(&self) -> Option<u64> {
        match self.0 {
            JsonValue::Int(int) => u64::try_from(*int).ok(),
            // jiter only produces a `BigInt` outside the `i64` range, which still leaves the top half of `u64`.
            JsonValue::BigInt(big) => u64::try_from(big).ok(),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self.0 {
            JsonValue::Int(int) => Some(*int),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self.0 {
            #[expect(
                clippy::cast_precision_loss,
                reason = "JSON Schema compares numbers as f64"
            )]
            JsonValue::Int(int) => Some(*int as f64),
            JsonValue::BigInt(big) => big.to_string().parse().ok(),
            JsonValue::Float(float) => Some(*float),
            _ => None,
        }
    }

    fn as_str(&self) -> Cow<'_, str> {
        match self.0 {
            JsonValue::BigInt(big) => Cow::Owned(big.to_string()),
            _ => Cow::Owned(self.to_number().to_string()),
        }
    }

    fn to_number(&self) -> Cow<'_, Number> {
        let number = match self.0 {
            JsonValue::Int(int) => Number::from(*int),
            // Without arbitrary precision, `serde_json` holds an integer past `u64` as a float, and so does this.
            JsonValue::BigInt(big) => match u64::try_from(big) {
                Ok(unsigned) => Number::from(unsigned),
                Err(_) => finite_number(
                    big.to_string()
                        .parse()
                        .expect("decimal digits parse as an f64"),
                ),
            },
            JsonValue::Float(float) => finite_number(*float),
            _ => unreachable!("a JiterNumber always holds a number"),
        };
        Cow::Owned(number)
    }

    fn is_integer(&self) -> bool {
        match self.0 {
            JsonValue::Int(_) | JsonValue::BigInt(_) => true,
            JsonValue::Float(float) => float.fract() == 0.0,
            _ => false,
        }
    }

    fn is_written_as_integer(&self) -> bool {
        // jiter parses a literal with a fraction or an exponent part as a `Float`, even `1.0` or `1e2`.
        matches!(self.0, JsonValue::Int(_) | JsonValue::BigInt(_))
    }
}

/// `number` as a `serde_json` number. jiter rejects `NaN` and `Infinity` when parsing, so every float it
/// produces is finite; a big integer too large for `f64` parses as an infinity, and saturates to the largest
/// finite float of its sign.
fn finite_number(number: f64) -> Number {
    Number::from_f64(number.clamp(f64::MIN, f64::MAX))
        .expect("a clamped, non-NaN f64 is a JSON number")
}

/// Copy a jiter value into a `serde_json` value, for error construction and keywords such as `enum`.
fn to_serde_value(value: &JsonValue<'_>) -> Value {
    match value {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(boolean) => Value::Bool(*boolean),
        JsonValue::Int(_) | JsonValue::BigInt(_) | JsonValue::Float(_) => {
            Value::Number(JiterNumber(value).to_number().into_owned())
        }
        JsonValue::Str(string) => Value::String(string.to_string()),
        JsonValue::Array(items) => Value::Array(items.iter().map(to_serde_value).collect()),
        // Inserting in document order lets a later duplicate replace an earlier one, as `serde_json` does.
        JsonValue::Object(members) => Value::Object(
            members
                .iter()
                .map(|(name, value)| (name.to_string(), to_serde_value(value)))
                .collect::<Map<String, Value>>(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use jsonschema::json::conformance;

    use super::*;

    #[test]
    fn jiter_representation_meets_the_validator_contract() {
        //* Given
        let text = conformance::document().to_string();
        let document = parse(&text).expect("the conformance document is JSON");

        //* Then
        conformance::assert_conformance::<Jiter>(&&document);
    }

    #[test]
    fn duplicated_key_validates_its_last_value() {
        //* Given
        let schema = serde_json::json!({"properties": {"id": {"type": "integer"}}});
        let validator = jsonschema::options_for::<Jiter>()
            .build(&schema)
            .expect("the schema compiles");
        let last_is_integer = parse(r#"{"id": "first", "id": 2}"#).expect("the instance is JSON");
        let last_is_string = parse(r#"{"id": 1, "id": "second"}"#).expect("the instance is JSON");

        //* Then
        assert!(validator.is_valid(&last_is_integer));
        assert!(!validator.is_valid(&last_is_string));
    }

    #[test]
    fn integer_past_u64_is_still_a_number() {
        //* Given
        let schema = serde_json::json!({"type": "integer", "minimum": 0});
        let validator = jsonschema::options_for::<Jiter>()
            .build(&schema)
            .expect("the schema compiles");
        let instance = parse("123456789012345678901234567890").expect("the instance is JSON");

        //* Then
        assert!(validator.is_valid(&instance));
    }
}
