use pyo3::prelude::*;
use pyo3::types::PyDict;

use std::str::FromStr;

use crate::errors::*;
use crate::policy::*;

/// Constructs a new `PolicySet` containing a copy of each _static_ policy in
/// `policy_set` with its ID set to the value of the annotation indicated by
/// `key` if it exists.
fn clone_policies_with_id_from_annotation_optional(
    policy_set: &cedar_policy::PolicySet,
    key: &str,
) -> cedar_policy::PolicySet {
    let mut new = cedar_policy::PolicySet::new();
    for policy in policy_set.policies() {
        if policy.is_static() {
            let policy = clone_policy_with_id_from_annotation_optional(policy, key);
            // add only raise errors when given a non static policy
            new.add(policy).unwrap();
        } else {
            // don't expect errors since policy set we iterate upon is valid
            new.link(
                policy.template_id().unwrap().clone(),
                policy.id().clone(),
                policy.template_links().unwrap(),
            )
            .unwrap();
        }
    }
    new
}

/// Represents a set of Policies.
///
/// Parameters:
///     policies: a list of policies
///
/// See also:
///     * <https://docs.rs/cedar-policy/latest/cedar_policy/struct.PolicySet.html>
#[pyclass(module = "cedar._lib")]
#[derive(Clone)]
pub struct PolicySet {
    pub policy_set: cedar_policy::PolicySet,
}

#[pymethods]
impl PolicySet {
    #[new]
    fn new_py(policies: Vec<Policy>) -> PyResult<Self> {
        cedar_policy::PolicySet::from_policies(policies.iter().map(|p| p.to_cedar_policy()))
            .or_value_error("failed to build policy set")
            .map(|policy_set| PolicySet { policy_set })
    }

    /// Create a policy set from a string in JSON policy format.
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
    #[pyo3(signature = (text, /))]
    fn from_json(text: &str) -> PyResult<PolicySet> {
        cedar_policy::PolicySet::from_json_str(text)
            .map(|policy_set| clone_policies_with_id_from_annotation_optional(&policy_set, "id"))
            .map(|policy_set| PolicySet { policy_set })
            .or_value_error("failed to parse policy set")
    }

    /// Create policy set from a Cedar policy string.
    ///
    /// Parameters:
    ///    text: a string holding policies in Cedar format
    ///
    /// Returns:
    ///     A policy set
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/syntax-policy.html>
    #[staticmethod]
    #[pyo3(signature = (text, /))]
    fn from_string(text: &str) -> PyResult<PolicySet> {
        cedar_policy::PolicySet::from_str(&text)
            .map(|policy_set| clone_policies_with_id_from_annotation_optional(&policy_set, "id"))
            .map(|policy_set| PolicySet { policy_set })
            .or_value_error("failed to parse policy set")
    }

    /// Create a policy set from a dict holding JSON policies definitions.
    ///
    /// Parameters:
    ///     values: a dict holding JSON policies definitions
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/json-format.html>
    #[staticmethod]
    #[pyo3(signature = (values, /))]
    fn from_dict(values: &Bound<'_, PyDict>) -> PyResult<Self> {
        pythonize::depythonize(values)
            .or_value_error("failed to parse dict")
            .and_then(|value| {
                cedar_policy::PolicySet::from_json_value(value)
                    .map(|policy_set| {
                        clone_policies_with_id_from_annotation_optional(&policy_set, "id")
                    })
                    .map(|policy_set| PolicySet { policy_set })
                    .or_value_error("failed to parse policy set")
            })
    }

    /// Serialize policy set into python dictionary.
    ///
    /// Returns:
    ///     A python dictionary
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/json-format.html>
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.policy_set
            .clone()
            .to_json()
            .or_value_error("failed to encode policy set to JSON")
            .and_then(|values| {
                pythonize::pythonize(py, &values)
                    .or_value_error("failed to serialize policy set to dict")
            })
    }

    /// Serialize policy set into JSON string.
    ///
    /// Returns:
    ///     A string in JSON policy format
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/json-format.html>
    fn to_json(&self) -> PyResult<String> {
        self.policy_set
            .clone()
            .to_json()
            .or_value_error("failed to encode policy set")
            .and_then(|value| {
                serde_json::to_string(&value)
                    .or_value_error("failed to serialize JSON policy set to string")
            })
    }

    /// Serialize policy set into Cedar policy string.
    ///
    /// Returns:
    ///     A string in Cedar policy format
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/syntax-policy.html>
    fn to_string(&self) -> String {
        self.policy_set.to_string()
    }

    /// Serialize policy set into formatted Cedar policy string.
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

    /// Get policies from the policy set
    ///
    /// Returns:
    ///     A list of policies
    #[getter]
    fn policies(&self) -> Vec<Policy> {
        self.policy_set
            .policies()
            .map(|policy| Policy::from_cedar_policy(policy.clone()))
            .collect()
    }
}
