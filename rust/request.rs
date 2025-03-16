use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};

use crate::{entity_uid::EntityUid, errors::IntoPyErr, schema::Schema};

/// Cedar authorization request.
///
/// Parameters:
///     principal: the principal to authorize
///     action: the action to authorize principal to perform
///     resource: the resource to authorize principal to take action on
///     context: the context for this auhtorization request
///
/// Tip:
///     An authorization request is a tuple <P, A, R, C> where
///
///     * P is the principal EntityUid,
///     * A is the action EntityUid,
///     * R is the resource EntityUid, and
///     * C is the request Context record.
///
///     It represents an authorization request asking the question, “Can this principal take this action on this resource in this context?”
#[pyclass(module = "cedar._lib")]
#[derive(Clone, Serialize, Deserialize)]
pub struct Request {
    /// Principal for the request, e.g., User::"alice"
    pub principal: EntityUid,
    /// Action for the request, e.g., Action::"view"
    pub action: EntityUid,
    /// Resource for the request, e.g., File::"myfile.txt"
    pub resource: EntityUid,
    /// A JSON string representing the context for the request.
    /// Should be a (possibly empty) map from keys to values.
    pub context: Option<serde_json::Value>,
    /// An optional correlation id that will be copied to the AuthResponse
    pub correlation_id: Option<String>,
}

impl Request {
    pub fn make_cedar_request(&self, schema: Option<&Schema>) -> PyResult<cedar_policy::Request> {
        let cedar_schema = schema.map(|s| &s.schema);
        // Validate context
        let cedar_context = match &self.context {
            Some(json) => cedar_policy::Context::from_json_value(
                json.clone(),
                cedar_schema.zip(Some(&self.action.make_cedar_euid()?)),
            )
            .or_value_error("failed to parse context")?,
            None => cedar_policy::Context::empty(),
        };
        // Make request
        cedar_policy::Request::new(
            self.principal.make_cedar_euid()?,
            self.action.make_cedar_euid()?,
            self.resource.make_cedar_euid()?,
            cedar_context,
            cedar_schema,
        )
        .or_value_error("failed to create request")
    }
}

#[pymethods]
impl Request {
    /// Create a new authorization request.
    #[new]
    #[pyo3(signature = (*, principal, action, resource, context = None, correlation_id = None))]
    fn new_py(
        principal: EntityUid,
        action: EntityUid,
        resource: EntityUid,
        context: Option<&Bound<'_, PyDict>>,
        correlation_id: Option<String>,
    ) -> PyResult<Request> {
        let context = match context {
            Some(context) => {
                let values: serde_json::Value = pythonize::depythonize(context)
                    .or_value_error("failed to parse context from dict")?;
                Some(values)
            }
            None => None,
        };
        Ok(Request {
            principal,
            action,
            resource,
            context,
            correlation_id: correlation_id.clone(),
        })
    }

    /// Create a new authorization request from a JSON string.
    #[staticmethod]
    fn from_json(text: &str) -> PyResult<Self> {
        serde_json::from_str(text).or_value_error("failed to parse request from JSON")
    }

    /// Create a new authorization request from a python dictionary.
    #[staticmethod]
    fn from_dict(values: &Bound<'_, PyDict>) -> PyResult<Self> {
        pythonize::depythonize(values)
            .or_value_error("failed to parse dict")
            .and_then(|json| serde_json::from_value(json).or_value_error("failed to build request"))
    }

    /// Get principal for this request.
    #[getter]
    fn principal(&self) -> EntityUid {
        self.principal.clone()
    }

    /// Get action for this request.
    #[getter]
    fn action(&self) -> EntityUid {
        self.action.clone()
    }

    /// Get resource for this request.
    #[getter]
    fn resource(&self) -> EntityUid {
        self.resource.clone()
    }

    /// Get the context for this request as a python dictionary.
    #[getter]
    fn context<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        match &self.context {
            Some(context) => pythonize::pythonize(py, &context)
                .or_value_error("failed to serialize context to dict")
                .map(|v| Some(v)),
            None => Ok(None),
        }
    }

    /// Get the correlation ID associated to this request.
    #[getter]
    fn correlation_id(&self) -> Option<String> {
        self.correlation_id.clone()
    }
}
