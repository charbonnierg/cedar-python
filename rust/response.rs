use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt, hash::Hash};

use crate::errors::IntoPyErr;
use pyo3::{prelude::*, types::PyDict};

/// Decision returned from the [Authorizer][cedar.Authorizer]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[pyclass(frozen, hash, eq, str, module = "cedar._lib")]
pub enum Decision {
    /// The [Authorizer][cedar.Authorizer] determined that the request should be denied.
    /// This is also returned if sufficiently fatal errors are encountered such
    /// that no decision could be safely reached; for example, errors parsing
    /// the policies.
    Deny,
    /// The [Authorizer][cedar.Authorizer] determined that the request should be allowed
    Allow,
}

impl fmt::Display for Decision {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        match self {
            Decision::Allow => write!(f, "allow"),
            Decision::Deny => write!(f, "deny"),
        }
    }
}

/// Authorization response returned from the [Authorizer][cedar.Authorizer].
///
/// Parameters:
///     decision: the authorization decision, either [Deny][cedar.Decision.Deny] or [Allow][cedar.Decision.Allow]
///     diagnostics: the authorization diagnostics
///     correlation_id: an optional correlation id as a string
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[pyclass(module = "cedar._lib")]
pub struct Response {
    decision: Decision,
    correlation_id: Option<String>,
    diagnostics: Diagnostics,
}

impl Response {
    pub fn from_cedar_response(
        response: cedar_policy::Response,
        correlation_id: Option<String>,
    ) -> Self {
        Self {
            decision: match response.decision() {
                cedar_policy::Decision::Allow => Decision::Allow,
                cedar_policy::Decision::Deny => Decision::Deny,
            },
            correlation_id,
            diagnostics: Diagnostics {
                reason: response.diagnostics().reason().cloned().collect(),
                errors: response
                    .diagnostics()
                    .errors()
                    .map(|e| match e {
                        cedar_policy::AuthorizationError::PolicyEvaluationError(e) => {
                            e.policy_id().to_string()
                        }
                    })
                    .collect(),
            },
        }
    }

    pub fn from_error(error: &str, correlation_id: Option<String>) -> Self {
        Self {
            decision: Decision::Deny,
            diagnostics: Diagnostics {
                reason: HashSet::new(),
                errors: vec![error.to_string()],
            },
            correlation_id,
        }
    }

    pub fn from_errors(errors: Vec<String>, correlation_id: Option<String>) -> Self {
        Self {
            decision: Decision::Deny,
            diagnostics: Diagnostics {
                reason: HashSet::new(),
                errors: errors,
            },
            correlation_id,
        }
    }
}
#[pymethods]
impl Response {
    #[new]
    #[pyo3(signature = (decision, diagnostics, correlation_id = None))]
    fn new_py(
        decision: Decision,
        diagnostics: Diagnostics,
        correlation_id: Option<String>,
    ) -> Self {
        return Response {
            decision,
            correlation_id,
            diagnostics,
        };
    }

    /// Create a new response from a JSON string.
    #[staticmethod]
    fn from_json(text: &str) -> PyResult<Self> {
        serde_json::from_str(text).or_value_error("failed to parse JSON")
    }

    /// Create a new response from a python dictionary.
    #[staticmethod]
    fn from_dict(values: &Bound<'_, PyDict>) -> PyResult<Self> {
        pythonize::depythonize(values)
            .or_value_error("failed to parse dict")
            .and_then(|json| serde_json::from_value(json).or_value_error("failed to parse dict"))
    }

    /// Serialize response to a python dictionary.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        pythonize::pythonize(py, self).or_value_error("failed to serialize to dict")
    }

    /// Serialize response to a JSON string.
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self).or_value_error("failed to serialize to JSON")
    }

    /// Get the decision from the response.
    #[getter]
    fn decision(&self) -> Decision {
        self.decision.clone()
    }

    /// Get diagnostics associated to decision.
    #[getter]
    fn diagnostics(&self) -> Diagnostics {
        self.diagnostics.clone()
    }

    /// Get the correlation ID which was provided in authorization request (may be None)
    #[getter]
    fn correlation_id(&self) -> Option<String> {
        self.correlation_id.clone()
    }
}

/// Diagnostics providing more information on how a [Decision][cedar.Decision] was reached.
///
/// Parameters:
///     reason: an optional list of policies ids
///     errors: an optional list of error messages
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[pyclass(module = "cedar._lib")]
pub struct Diagnostics {
    /// `PolicyId`s of the policies that contributed to the decision.
    /// If no policies applied to the request, this set will be empty.
    reason: HashSet<cedar_policy::PolicyId>,
    /// Errors that occurred during authorization. The errors should be
    /// treated as unordered, since policies may be evaluated in any order.
    errors: Vec<String>,
}

#[pymethods]
impl Diagnostics {
    #[new]
    #[pyo3(signature = (reason = None, errors = None))]
    fn new_py(reason: Option<Vec<String>>, errors: Option<Vec<String>>) -> Self {
        Diagnostics {
            reason: reason
                .map(|reason| {
                    reason
                        .iter()
                        .map(|r: &String| cedar_policy::PolicyId::new(r))
                        .collect()
                })
                .unwrap_or(HashSet::new()),
            errors: errors.unwrap_or(vec![]),
        }
    }

    /// Create a new diagnostics instance from a JSON string.
    #[staticmethod]
    fn from_json(text: &str) -> PyResult<Self> {
        serde_json::from_str(text).or_value_error("failed to parse JSON")
    }

    /// Create a new diagnostics instance from a python dictionary.
    #[staticmethod]
    fn from_dict(values: &Bound<'_, PyDict>) -> PyResult<Self> {
        pythonize::depythonize(values)
            .or_value_error("failed to parse dict")
            .and_then(|json| serde_json::from_value(json).or_value_error("failed to parse dict"))
    }

    /// Serialize diagnotics to python dictionary.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        pythonize::pythonize(py, self).or_value_error("failed to serialize to dict")
    }

    /// Serialize diagnostics to JSON string.
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self).or_value_error("failed to serialize to JSON")
    }

    /// Get the PolicyIds of the policies that contributed to the decision.
    /// If no policies applied to the request, this set will be empty.
    #[getter]
    fn reasons(&self) -> HashSet<String> {
        self.reason.iter().map(|r| r.to_string()).collect()
    }

    /// Get the errors that occurred during authorization.
    /// The errors should be treated as unordered, since policies may be evaluated in any order.
    #[getter]
    fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }
}
