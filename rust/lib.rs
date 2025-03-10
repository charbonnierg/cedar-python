use pyo3::prelude::*;
use std::collections::HashSet;
use std::io::BufWriter;
use std::iter;
use std::str::FromStr;

// Pretty-print policy in text format.
#[pyfunction]
#[pyo3(signature = (s, /, *, line_width, indent_width))]
fn format_policies(s: String, line_width: usize, indent_width: isize) -> PyResult<String> {
    let config = cedar_policy_formatter::Config {
        line_width,
        indent_width,
    };

    match cedar_policy_formatter::policies_str_to_pretty(&s, &config) {
        Ok(s) => Ok(s),
        Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
    }
}

// Cedar policies
#[pyclass]
#[derive(Clone)]
struct Policies {
    _policy_set: cedar_policy::PolicySet,
}

#[pymethods]
impl Policies {
    // Create new Policies from a JSON string.
    #[staticmethod]
    fn from_json(s: String) -> PyResult<Policies> {
        match cedar_policy::PolicySet::from_json_str(s) {
            Ok(p) => Ok(Policies { _policy_set: p }),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }

    // Create new policies from a string.
    #[staticmethod]
    fn from_string(s: String) -> PyResult<Policies> {
        match cedar_policy::PolicySet::from_str(&s) {
            Ok(p) => Ok(Policies { _policy_set: p }),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }

    // Serialize policies to a JSON string.
    fn to_json(&self) -> PyResult<String> {
        match self._policy_set.clone().to_json() {
            Ok(s) => Ok(s.to_string()),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }

    // Serialize policies to a string.
    fn to_string(&self) -> PyResult<String> {
        Ok(self._policy_set.to_string())
    }

    #[pyo3(signature = (line_width = None, indent_width = None))]
    fn to_pretty_string(
        &self,
        line_width: Option<usize>,
        indent_width: Option<isize>,
    ) -> PyResult<String> {
        let config = cedar_policy_formatter::Config {
            line_width: line_width.unwrap_or(88),
            indent_width: indent_width.unwrap_or(2),
        };
        match cedar_policy_formatter::policies_str_to_pretty(&self._policy_set.to_string(), &config)
        {
            Ok(s) => Ok(s),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }
}

// Cedar entities
#[pyclass]
#[derive(Clone)]
struct Entities {
    _entities: cedar_policy::Entities,
}

#[pymethods]
impl Entities {
    // Parse entities from JSON, optionally validating using schema.
    #[staticmethod]
    #[pyo3(signature = (s, schema=None))]
    fn from_json(s: String, schema: Option<Schema>) -> PyResult<Entities> {
        match cedar_policy::Entities::from_json_str(&s, schema.as_ref().map(|s| &s._schema)) {
            Ok(entities) => Ok(Entities {
                _entities: entities,
            }),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }

    // Format entities to JSON.
    fn to_json(&self) -> PyResult<String> {
        let mut buf = BufWriter::new(Vec::new());
        let result = self._entities.write_to_json(&mut buf);
        match result {
            Ok(_) => match buf.into_inner() {
                Ok(vec) => match String::from_utf8(vec) {
                    Ok(s) => Ok(s),
                    Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
                },
                Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
            },
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }

    // Format entities to dot format string.
    fn to_string(&self) -> PyResult<String> {
        Ok(self._entities.to_dot_str())
    }
}

// Output of schema validation
#[pyclass]
struct SchemaValidationResult {
    _passed: bool,
    _passwed_without_warning: bool,
    _errors: Vec<String>,
    _warnings: Vec<String>,
    _msg: String,
}

#[pymethods]
impl SchemaValidationResult {
    #[getter]
    fn passed(&self) -> bool {
        return self._passed;
    }
    #[getter]
    fn passed_without_warning(&self) -> bool {
        return self._passwed_without_warning;
    }
    #[getter]
    fn errors(&self) -> Vec<String> {
        return self._errors.clone();
    }
    #[getter]
    fn warnings(&self) -> Vec<String> {
        return self._warnings.clone();
    }
    fn to_string(&self) -> String {
        return self._msg.clone();
    }
}

// Output of entities validation
#[pyclass]
struct EntitiesValidationResult {
    _passed: bool,
    _error: String,
    _entities: Entities,
}

#[pymethods]
impl EntitiesValidationResult {
    #[getter]
    fn passed(&self) -> bool {
        return self._passed;
    }
    #[getter]
    fn error(&self) -> String {
        return self._error.clone();
    }
    #[getter]
    fn entities(&self) -> Entities {
        return self._entities.clone();
    }
}

// Cedar schema
#[pyclass]
#[derive(Clone)]
struct Schema {
    _fragment: cedar_policy::SchemaFragment,
    _schema: cedar_policy::Schema,
    _validator: cedar_policy::Validator,
}

#[pymethods]
impl Schema {
    #[staticmethod]
    fn from_json(s: String) -> PyResult<Schema> {
        match cedar_policy::SchemaFragment::from_json_str(&s) {
            Ok(fragment) => {
                match cedar_policy::Schema::from_schema_fragments(iter::once(fragment.clone())) {
                    Ok(schema) => Ok(Schema {
                        _fragment: fragment,
                        _schema: schema.clone(),
                        _validator: cedar_policy::Validator::new(schema),
                    }),
                    Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
                }
            }
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }
    #[staticmethod]
    fn from_string(s: String) -> PyResult<Schema> {
        match cedar_policy::SchemaFragment::from_json_str(&s) {
            Ok(fragment) => match cedar_policy::Schema::from_str(&s) {
                Ok(schema) => Ok(Schema {
                    _fragment: fragment,
                    _schema: schema.clone(),
                    _validator: cedar_policy::Validator::new(schema),
                }),
                Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
            },
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }

    fn to_json(&self) -> PyResult<String> {
        match self._fragment.to_json_string() {
            Ok(s) => Ok(s),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }

    fn to_string(&self) -> PyResult<String> {
        match self._fragment.to_cedarschema() {
            Ok(s) => Ok(s),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }

    fn validate_policies(&self, policies: Policies) -> SchemaValidationResult {
        let result = self
            ._validator
            .validate(&policies._policy_set, cedar_policy::ValidationMode::Strict);
        SchemaValidationResult {
            _passed: result.validation_passed(),
            _passwed_without_warning: result.validation_passed_without_warnings(),
            _errors: result.validation_errors().map(|e| e.to_string()).collect(),
            _warnings: result
                .validation_warnings()
                .map(|w| w.to_string())
                .collect(),
            _msg: result.to_string(),
        }
    }

    fn validate_entities(&self, entities: Entities) -> EntitiesValidationResult {
        match cedar_policy::Entities::from_entities(entities._entities, Some(&self._schema)) {
            Ok(_entities) => EntitiesValidationResult {
                _passed: true,
                _error: String::new(),
                _entities: Entities { _entities },
            },
            Err(e) => EntitiesValidationResult {
                _passed: false,
                _error: e.to_string(),
                _entities: Entities {
                    _entities: cedar_policy::Entities::empty(),
                },
            },
        }
    }
}

// Cedar authorization request
#[pyclass]
#[derive(Clone)]
struct Request {
    /// Principal for the request, e.g., User::"alice"
    principal: cedar_policy::EntityUid,
    /// Action for the request, e.g., Action::"view"
    action: cedar_policy::EntityUid,
    /// Resource for the request, e.g., File::"myfile.txt"
    resource: cedar_policy::EntityUid,
    /// A JSON object representing the context for the request.
    /// Should be a (possibly empty) map from keys to values.
    context: cedar_policy::Context,
    /// An optional correlation id that will be copied to the AuthResponse
    correlation_id: Option<String>,
}

#[pymethods]
impl Request {
    #[staticmethod]
    #[pyo3(signature = (principal, action, resource, context, correlation_id=None))]
    fn new(
        principal: String,
        action: String,
        resource: String,
        context: Option<String>,
        correlation_id: Option<String>,
    ) -> PyResult<Request> {
        let ctx = match context {
            Some(s) => cedar_policy::Context::from_json_str(&s, None),
            None => Ok(cedar_policy::Context::empty()),
        };
        if ctx.is_err() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                ctx.unwrap_err().to_string(),
            ));
        }
        let ctx = ctx.unwrap();
        match cedar_policy::EntityUid::from_str(&principal) {
            Ok(p) => match cedar_policy::EntityUid::from_str(&action) {
                Ok(a) => match cedar_policy::EntityUid::from_str(&resource) {
                    Ok(r) => Ok(Request {
                        principal: p,
                        action: a,
                        resource: r,
                        context: ctx,
                        correlation_id: correlation_id.clone(),
                    }),
                    Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
                },
                Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
            },
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }
}

// Cedar authorizer
#[pyclass]
struct Authorizer {
    policies: Policies,
    schema: Option<Schema>,
}

#[pymethods]
impl Authorizer {
    #[staticmethod]
    #[pyo3(signature = (policies=None, schema=None))]
    fn new(policies: Option<Policies>, schema: Option<Schema>) -> PyResult<Authorizer> {
        // Make policies
        let policies = Policies {
            _policy_set: policies
                .map(|p| p._policy_set)
                .or(Some(cedar_policy::PolicySet::new()))
                .unwrap(),
        };
        // Optionally validate schema using validator
        match &schema {
            Some(s) => {
                let result = s.validate_policies(policies.clone());
                if !result.passed() {
                    return Err(pyo3::exceptions::PyValueError::new_err(result.to_string()));
                }
            }
            None => (),
        }
        // Make authorizer
        Ok(Authorizer { policies, schema })
    }

    #[pyo3(signature = (schema=None))]
    fn with_schema(&self, schema: Option<Schema>) -> PyResult<Authorizer> {
        return Authorizer::new(Some(self.policies.clone()), schema);
    }

    #[pyo3(signature = (policies=None))]
    fn with_policies(&self, policies: Option<Policies>) -> PyResult<Authorizer> {
        return Authorizer::new(policies, self.schema.clone());
    }

    fn is_authorized(&self, request: &Request, entities: Entities) -> PyResult<Response> {
        let schema = self.schema.as_ref().map(|s| &s._schema);
        let cedar_entities = match cedar_policy::Entities::from_entities(entities._entities, schema)
        {
            Ok(v) => v,
            Err(e) => {
                return Err(pyo3::exceptions::PyValueError::new_err(e.to_string()));
            }
        };
        match cedar_policy::Request::new(
            request.principal.clone(),
            request.action.clone(),
            request.resource.clone(),
            request.context.clone(),
            schema,
        ) {
            Ok(req) => {
                let authorizer = cedar_policy::Authorizer::new();
                let response =
                    authorizer.is_authorized(&req, &self.policies._policy_set, &cedar_entities);
                Ok(Response::new(response, request.correlation_id.clone()))
            }
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[pyclass]
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
    #[getter]
    fn reasons(&self) -> Vec<String> {
        self.reason.iter().map(|r| r.to_string()).collect()
    }
    #[getter]
    fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[pyclass(eq, eq_int)]
pub enum Decision {
    /// The `Authorizer` determined that the request should be allowed
    Allow,
    /// The `Authorizer` determined that the request should be denied.
    /// This is also returned if sufficiently fatal errors are encountered such
    /// that no decision could be safely reached; for example, errors parsing
    /// the policies.
    Deny,
}

/// Authorization response returned from the `Authorizer`
#[derive(Debug, PartialEq, Clone)]
#[pyclass]
struct Response {
    /// Authorization decision
    decision: Decision,

    /// (Optional) id to correlate this response to the request
    correlation_id: Option<String>,

    /// Diagnostics providing more information on how this decision was reached
    diagnostics: Diagnostics,
}

impl Response {
    /// Create a new `AuthzResponse`
    pub fn new(response: cedar_policy::Response, correlation_id: Option<String>) -> Self {
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
                    .cloned()
                    .map(|e| e.to_string())
                    .collect(),
            },
        }
    }
}
#[pymethods]
impl Response {
    #[getter]
    pub fn decision(&self) -> Decision {
        self.decision.clone()
    }
    #[getter]
    pub fn correlation_id(&self) -> Option<String> {
        self.correlation_id.clone()
    }
    #[getter]
    pub fn diagnostics(&self) -> Diagnostics {
        self.diagnostics.clone()
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn _lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Entities>()?;
    m.add_class::<Policies>()?;
    m.add_class::<Schema>()?;
    m.add_class::<Authorizer>()?;
    m.add_class::<Request>()?;
    m.add_class::<Decision>()?;
    m.add_class::<Diagnostics>()?;
    m.add_class::<Response>()?;
    m.add_function(wrap_pyfunction!(format_policies, m)?)?;
    Ok(())
}
