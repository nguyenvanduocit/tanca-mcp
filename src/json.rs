//! JSON helpers that keep the Go implementation's decoding and encoding rules.

use serde::de::DeserializeOwned;
use serde::{Serialize, Serializer};
use serde_json::Value;

/// Removes `null` members from every object, recursively.
///
/// Combined with `#[serde(default)]`, this decodes `null` to the zero value
/// (as Go's `encoding/json` does for non-pointer fields) and to `None` for
/// `Option` fields.
pub fn strip_nulls(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, v| !v.is_null());
            map.values_mut().for_each(strip_nulls);
        }
        Value::Array(items) => items.iter_mut().for_each(strip_nulls),
        _ => {}
    }
}

/// Decodes `value` into `T` after [`strip_nulls`].
pub fn from_value<T: DeserializeOwned>(mut value: Value) -> serde_json::Result<T> {
    strip_nulls(&mut value);
    serde_json::from_value(value)
}

/// Decodes a JSON byte slice into `T` after [`strip_nulls`].
pub fn from_slice<T: DeserializeOwned>(body: &[u8]) -> serde_json::Result<T> {
    from_value(serde_json::from_slice(body)?)
}

/// Serializes `value` as two-space indented JSON with a trailing newline,
/// escaping `<`, `>`, `&`, U+2028 and U+2029 like Go's `json.Encoder`.
pub fn to_pretty_string<T: Serialize + ?Sized>(value: &T) -> serde_json::Result<String> {
    let raw = serde_json::to_string_pretty(value)?;
    let mut out = String::with_capacity(raw.len() + 1);
    // These characters can only appear inside JSON strings, so escaping them
    // in the serialized text is safe.
    for c in raw.chars() {
        match c {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c => out.push(c),
        }
    }
    out.push('\n');
    Ok(out)
}

pub(crate) fn is_zero_i64(v: &i64) -> bool {
    *v == 0
}

pub(crate) fn is_zero_f64(v: &f64) -> bool {
    *v == 0.0
}

/// Serializes whole floats without a fractional part (`21` rather than
/// `21.0`), matching Go's encoding of `float64`.
pub(crate) fn go_float<S: Serializer>(v: &f64, s: S) -> Result<S::Ok, S::Error> {
    if v.fract() == 0.0 && v.abs() < 1e21 {
        s.serialize_i64(*v as i64)
    } else {
        s.serialize_f64(*v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct S {
        a: String,
        b: Option<String>,
        n: i64,
    }

    #[test]
    fn nulls_decode_to_zero_values() {
        let s: S = from_slice(br#"{"a":null,"b":null,"n":null}"#).unwrap();
        assert_eq!(s.a, "");
        assert_eq!(s.b, None);
        assert_eq!(s.n, 0);
    }

    #[test]
    fn pretty_output_escapes_html() {
        let v = serde_json::json!({"x": "<a&b>"});
        assert_eq!(
            to_pretty_string(&v).unwrap(),
            "{\n  \"x\": \"\\u003ca\\u0026b\\u003e\"\n}\n"
        );
    }
}
