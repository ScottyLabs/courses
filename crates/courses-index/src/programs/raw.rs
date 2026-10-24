//! Reading Stellic's JSON the way `build-programs.ts` does: property access that yields `undefined`, `??`,
//! truthiness, `Number()` and `String()`. Courseweave reads the export untyped, so the port keeps its coercions.

use serde::{Serialize, Serializer};
use serde_json::Value;

use crate::fce::js::{self, Num};

/// A JavaScript value read from parsed JSON; `None` is `undefined`.
#[derive(Clone, Copy, Debug)]
pub struct Js<'a>(pub Option<&'a Value>);

impl<'a> Js<'a> {
    pub fn new(value: &'a Value) -> Self {
        Self(Some(value))
    }

    /// `x.key` (or `x?.key`): `undefined` unless `x` is an object with the key.
    pub fn get(self, key: &str) -> Js<'a> {
        Js(self.0.and_then(Value::as_object).and_then(|o| o.get(key)))
    }

    /// `x[i]` on an array.
    pub fn at(self, i: usize) -> Js<'a> {
        Js(self.0.and_then(Value::as_array).and_then(|a| a.get(i)))
    }

    pub fn is_nullish(self) -> bool {
        matches!(self.0, None | Some(Value::Null))
    }

    /// `self ?? other`.
    pub fn or(self, other: Js<'a>) -> Js<'a> {
        if self.is_nullish() { other } else { self }
    }

    /// `Boolean(x)`.
    pub fn truthy(self) -> bool {
        match self.0 {
            None | Some(Value::Null) => false,
            Some(Value::Bool(b)) => *b,
            Some(Value::Number(n)) => n.as_f64().is_some_and(|x| x != 0.0 && !x.is_nan()),
            Some(Value::String(s)) => !s.is_empty(),
            Some(Value::Array(_) | Value::Object(_)) => true,
        }
    }

    /// `Number(x)`.
    pub fn number(self) -> f64 {
        match self.0 {
            None => f64::NAN,
            Some(Value::Null) => 0.0,
            Some(Value::Bool(b)) => f64::from(u8::from(*b)),
            Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
            Some(Value::String(s)) => js::string_to_number(s),
            Some(Value::Array(_)) => js::string_to_number(&self.string()),
            Some(Value::Object(_)) => f64::NAN,
        }
    }

    /// `Number(x ?? default)`.
    pub fn number_or(self, default: f64) -> f64 {
        if self.is_nullish() {
            default
        } else {
            self.number()
        }
    }

    /// `String(x)`.
    pub fn string(self) -> String {
        match self.0 {
            None => "undefined".into(),
            Some(v) => value_string(v),
        }
    }

    /// The string, for places Courseweave calls string methods on the value. Anything else reads as `""`.
    pub fn str(self) -> &'a str {
        self.0.and_then(Value::as_str).unwrap_or("")
    }

    /// The items of `x ?? []`. Values that aren't arrays have none.
    pub fn items(self) -> impl Iterator<Item = Js<'a>> {
        self.0
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(Js::new)
    }

    /// Whether `x.length` is truthy: a non-empty array or string.
    pub fn has_length(self) -> bool {
        match self.0 {
            Some(Value::Array(a)) => !a.is_empty(),
            Some(Value::String(s)) => !s.is_empty(),
            _ => false,
        }
    }

    /// An integral number, as a catalog or audit id (`Map` keys compare numbers by value).
    pub fn id(self) -> Option<i64> {
        let x = self.0?.as_f64()?;
        #[expect(clippy::cast_possible_truncation, reason = "checked integral")]
        (x.fract() == 0.0 && x.abs() < 9_007_199_254_740_992.0).then_some(x as i64)
    }

    /// The value as `JSON.stringify` writes it inside an array (`undefined` as `null`).
    pub fn to_raw(self) -> Raw {
        Raw(self.0.cloned().unwrap_or(Value::Null))
    }

    /// `a === b` for values from JSON: by value for primitives, never for distinct objects.
    pub fn strict_eq(self, other: Js) -> bool {
        match (self.0, other.0) {
            (None, None) => true,
            (Some(Value::Null), Some(Value::Null)) => true,
            (Some(Value::Bool(a)), Some(Value::Bool(b))) => a == b,
            (Some(Value::Number(a)), Some(Value::Number(b))) => a.as_f64() == b.as_f64(),
            (Some(Value::String(a)), Some(Value::String(b))) => a == b,
            _ => false,
        }
    }
}

fn value_string(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => js::number_to_string(n.as_f64().unwrap_or(f64::NAN)),
        Value::String(s) => s.clone(),
        Value::Array(a) => a
            .iter()
            .map(|x| match x {
                Value::Null => String::new(),
                x => value_string(x),
            })
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".into(),
    }
}

/// A value copied from the export, serialized with JavaScript's number formatting (`9.0` as `9`).
#[derive(Clone, Debug, PartialEq)]
pub struct Raw(pub Value);

impl Raw {
    pub fn js(&self) -> Js<'_> {
        Js::new(&self.0)
    }
}

impl Serialize for Raw {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        RawRef(&self.0).serialize(s)
    }
}

struct RawRef<'a>(&'a Value);

impl Serialize for RawRef<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Value::Number(n) => Num(n.as_f64().unwrap_or(f64::NAN)).serialize(s),
            Value::Array(a) => s.collect_seq(a.iter().map(RawRef)),
            Value::Object(o) => s.collect_map(o.iter().map(|(k, v)| (k, RawRef(v)))),
            other => other.serialize(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn coerces_like_javascript() {
        let v = json!({ "x": "3", "n": null, "f": 9.0, "a": [1, null, 2], "e": "", "o": {} });
        let js = Js::new(&v);
        assert_eq!(js.get("x").number(), 3.0);
        assert_eq!(js.get("n").number(), 0.0);
        assert!(js.get("missing").number().is_nan());
        assert_eq!(js.get("missing").number_or(1.0), 1.0);
        assert_eq!(js.get("n").number_or(1.0), 1.0);
        assert_eq!(js.get("f").string(), "9");
        assert_eq!(js.get("a").string(), "1,,2");
        assert_eq!(js.get("missing").string(), "undefined");
        assert!(!js.get("e").truthy());
        assert!(js.get("o").truthy());
        assert_eq!(js.get("f").id(), Some(9));
        assert_eq!(js.get("x").id(), None);
        assert!(js.get("missing").or(js.get("x")).strict_eq(js.get("x")));
        assert_eq!(serde_json::to_string(&js.get("f").to_raw()).unwrap(), "9");
        assert_eq!(
            serde_json::to_string(&Js::new(&json!({"a": [1.0, 2.5]})).to_raw()).unwrap(),
            r#"{"a":[1,2.5]}"#
        );
    }
}
