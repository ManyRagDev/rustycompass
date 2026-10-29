use crate::{codec, model};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub &'static str);
pub type Result<T> = std::result::Result<T, Error>;
impl Error {
    pub fn exit_code(&self) -> i32 {
        match self.0 {
            "invalid_arguments" | "incompatible_inputs" => 2,
            "cancelled" => 130,
            _ => 1,
        }
    }
    pub fn value(&self) -> Value {
        json!({"error":{"code":self.0,"message":self.0}})
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for Error {}
pub(crate) fn canon<T: serde::Serialize>(v: &T) -> Result<Vec<u8>> {
    codec::canonical(v).map_err(|_| Error("input_invalid"))
}
pub(crate) fn number(n: u128) -> Result<String> {
    u64::try_from(n)
        .map(|n| n.to_string())
        .map_err(|_| Error("arithmetic_overflow"))
}
pub(crate) fn uint(v: &Value) -> Result<u64> {
    v.as_str()
        .ok_or(Error("input_invalid"))
        .and_then(|s| model::decimal(s).map_err(|_| Error("input_invalid")))
}
#[derive(Clone, Debug)]
pub struct Limits {
    pub(crate) values: BTreeMap<String, u64>,
    pub(crate) duration: Option<u64>,
}
impl Default for Limits {
    fn default() -> Self {
        let policy: Value = serde_json::from_str(include_str!(
            "../../../../research/proposals/d0-c2/fixtures/metric-policy.json"
        ))
        .expect("frozen policy");
        Self {
            values: policy["limits"]
                .as_object()
                .unwrap()
                .iter()
                .filter(|(k, _)| k.as_str() != "max_duration_ms")
                .map(|(k, v)| (k.clone(), v.as_str().unwrap().parse().unwrap()))
                .collect(),
            duration: None,
        }
    }
}
impl Limits {
    pub fn get(&self, name: &str) -> u64 {
        self.values[name]
    }
    pub fn reduced(profile: Option<&Value>, duration: Option<u64>) -> Result<Self> {
        let mut limits = Self::default();
        if duration.is_some_and(|n| n == 0 || n > 3_600_000) {
            return Err(Error("invalid_arguments"));
        }
        if let Some(profile) = profile {
            let object = profile.as_object().ok_or(Error("invalid_arguments"))?;
            if object.len() != limits.values.len()
                || object.keys().any(|k| !limits.values.contains_key(k))
            {
                return Err(Error("invalid_arguments"));
            }
            for (k, default) in limits.values.iter_mut() {
                let n = uint(&profile[k]).map_err(|_| Error("invalid_arguments"))?;
                if n == 0 || n > *default {
                    return Err(Error("invalid_arguments"));
                }
                *default = n;
            }
        }
        limits.duration = duration;
        Ok(limits)
    }
    pub(crate) fn policy(&self) -> Value {
        let mut policy: Value = serde_json::from_str(include_str!(
            "../../../../research/proposals/d0-c2/fixtures/metric-policy.json"
        ))
        .expect("frozen policy");
        for (k, v) in &self.values {
            policy["limits"][k] = json!(v.to_string());
        }
        policy["limits"]["max_duration_ms"] = self
            .duration
            .map(|n| json!(n.to_string()))
            .unwrap_or(Value::Null);
        policy
    }
}
#[derive(Clone, Debug)]
pub struct Request {
    pub baseline: PathBuf,
    pub current: PathBuf,
    pub baseline_source: Option<PathBuf>,
    pub current_source: Option<PathBuf>,
    pub limits: Limits,
}
#[derive(Debug)]
pub struct Output {
    pub value: Value,
    pub bytes: Vec<u8>,
    pub exit_code: i32,
}
