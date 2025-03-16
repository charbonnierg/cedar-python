use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::fmt::{self, Debug};
use std::str::FromStr;

use crate::errors::*;

/// Clones the provided policy with its ID set to the value of the annotation
/// indicated by `key` if it exists.
pub fn clone_policy_with_id_from_annotation_optional(
    policy: &cedar_policy::Policy,
    key: &str,
) -> cedar_policy::Policy {
    policy
        .annotation(key)
        .map(|value| policy.new_id(cedar_policy::PolicyId::new(value)))
        .unwrap_or(policy.clone())
}

/// Cedar policy
///
/// See also:
///     * <https://docs.cedarpolicy.com/policies/syntax-policy.html>
///     * <https://docs.cedarpolicy.com/policies/policy-examples.html>
///     * <https://docs.rs/cedar-policy/latest/cedar_policy/struct.Policy.html>
#[pyclass(module = "cedar._lib")]
#[derive(Clone)]
pub struct Policy {
    policy: cedar_policy::Policy,
}

impl Policy {
    pub fn from_cedar_policy(policy: cedar_policy::Policy) -> Self {
        Policy { policy }
    }
    pub fn to_cedar_policy(&self) -> cedar_policy::Policy {
        self.policy.clone()
    }
}

#[pymethods]
impl Policy {
    /// Create a policy from a string in Cedar policy format.
    ///
    /// Parameters:
    ///     text: a string in Cedar policy format
    ///
    /// Returns:
    ///     A policy
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/syntax-policy.html>
    #[staticmethod]
    #[pyo3(signature = (text, id = None))]
    fn from_string(text: &str, id: Option<String>) -> PyResult<Self> {
        match id {
            Some(id) => cedar_policy::Policy::from_str(&text).map(|policy| {
                policy.new_id(
                    cedar_policy::PolicyId::from_str(&id).unwrap_or_else(|never| match never {}),
                )
            }),
            None => cedar_policy::Policy::from_str(&text)
                .map(|policy| clone_policy_with_id_from_annotation_optional(&policy, "id")),
        }
        .map(|policy| Policy { policy })
        .or_value_error("failed to parse policy from string")
    }

    /// Create a policy from a string in JSON policy format.
    ///
    /// Parameters:
    ///     text: a string in JSON policy format
    ///
    /// Returns:
    ///     A policy
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/json-format.html>
    #[staticmethod]
    #[pyo3(signature = (text, id = None))]
    fn from_json(text: &str, id: Option<String>) -> PyResult<Self> {
        serde_json::from_str(&text)
            .or_value_error("failed to deserialize JSON")
            .and_then(|value| {
                cedar_policy::Policy::from_json(
                    id.map(|v| {
                        cedar_policy::PolicyId::from_str(&v).unwrap_or_else(|never| match never {})
                    }),
                    value,
                )
                .or_value_error("failed to parse policy from JSON")
            })
            .map(|policy| Policy { policy })
    }

    /// Create a policy from a dict holding JSON policy definition.
    ///
    /// Parameters:
    ///     values: a dict holding JSON policy definition
    ///
    /// Returns:
    ///     A policy
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/json-format.html>
    #[staticmethod]
    #[pyo3(signature = (values, /, *, id = None))]
    fn from_dict(values: &Bound<'_, PyDict>, id: Option<String>) -> PyResult<Self> {
        pythonize::depythonize(values)
            .or_value_error("failed to parse dict")
            .and_then(|value| {
                cedar_policy::Policy::from_json(
                    id.map(|v| {
                        cedar_policy::PolicyId::from_str(&v).unwrap_or_else(|never| match never {})
                    }),
                    value,
                )
                .or_value_error("failed to parse json value")
            })
            .map(|policy| Policy { policy })
    }

    /// Serialize policy into python dictionary.
    ///
    /// Returns:
    ///     A python dictionary
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/json-format.html>
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.policy
            .to_json()
            .or_value_error("failed to encode policy to JSON")
            .and_then(|values| {
                pythonize::pythonize(py, &values).or_value_error("failed to serialize to dict")
            })
    }

    /// Serialize policy into JSON string.
    ///
    /// Returns:
    ///     A string in JSON policy format
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/json-format.html>
    fn to_json(&self) -> PyResult<String> {
        self.policy
            .to_json()
            .or_value_error("failed to encode policy to JSON")
            .and_then(|value| {
                serde_json::to_string(&value)
                    .or_value_error("failed to serialize JSON policy to string")
            })
    }

    /// Serialize policy into Cedar policy string.
    ///
    /// Returns:
    ///     A string in Cedar policy format
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/syntax-policy.html>
    fn to_string(&self) -> String {
        self.policy.to_string()
    }

    /// Serialize policy into formatted Cedar policy string.
    ///
    /// By default, formatter uses a line width of 88 and an indentation width
    /// of 2.
    ///
    /// Parameters:
    ///     line_width: an optional integer configuring formatter line width
    ///     indent_width: an optional integer configuring formatter indentation width
    ///
    /// Returns:
    ///     A string in Cedar policy format
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/syntax-policy.html>
    #[pyo3(signature = (*, line_width = None, indent_width = None))]
    fn to_pretty_string(
        &self,
        line_width: Option<usize>,
        indent_width: Option<isize>,
    ) -> PyResult<String> {
        let config = cedar_policy_formatter::Config {
            line_width: line_width.unwrap_or(88),
            indent_width: indent_width.unwrap_or(2),
        };
        cedar_policy_formatter::policies_str_to_pretty(&self.to_string(), &config)
            .or_value_error("failed to format policy set")
    }

    /// Get policy effect.
    ///
    /// Returns:
    ///     The policy effect.
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/syntax-policy.html#term-policy-effect>
    #[getter]
    fn effect(&self) -> Effect {
        match self.policy.effect() {
            cedar_policy::Effect::Forbid => Effect::Forbid,
            cedar_policy::Effect::Permit => Effect::Permit,
        }
    }

    /// Get policy ID as string.
    ///
    /// Returns:
    ///     The policy ID as string
    #[getter]
    fn policy_id(&self) -> String {
        self.policy.id().to_string()
    }
}

/// An effect specifies the intent of a policy, to either permit or forbid
/// any request that matches the scope and conditions specified in the policy.
///
/// See also:
///     * <https://docs.cedarpolicy.com/policies/syntax-policy.html#term-policy-effect>
#[pyclass(eq, frozen, hash, str, module = "cedar._lib")]
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum Effect {
    /// If all elements in the policy match, then the policy results in a Deny decision.
    Forbid,
    /// If all elements in the policy match, then the policy results in an Allow decision.
    Permit,
}

impl fmt::Display for Effect {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        match self {
            Effect::Permit => write!(f, "permit"),
            Effect::Forbid => write!(f, "forbid"),
        }
    }
}
