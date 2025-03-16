use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::Deserialize;
use serde::Serialize;

use std::iter;
use std::str::FromStr;

use crate::errors::*;
use crate::policy_set::*;

#[pyclass(eq, frozen, hash, module = "cedar._lib")]
#[derive(PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct ValidationError {
    pub policy_id: String,
    pub error: String,
}

#[pyclass(eq, frozen, hash, module = "cedar._lib")]
#[derive(PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub policy_id: String,
    pub warning: String,
}

/// Output of policy validation against a schema.
///
/// Contains the result of policy validation.
/// The result includes the list of issues found by validation and whether validation succeeds or fails.
/// Validation succeeds if there are no fatal errors.
/// There may still be non-fatal warnings present when validation passes.
#[pyclass(eq, frozen, hash, module = "cedar._lib")]
#[derive(PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub passwed_without_warning: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub msg: String,
}

impl ValidationResult {
    fn from_cedar_validation_result(result: &cedar_policy::ValidationResult) -> Self {
        ValidationResult {
            passed: result.validation_passed(),
            passwed_without_warning: result.validation_passed_without_warnings(),
            errors: result
                .validation_errors()
                .map(|e| ValidationError {
                    policy_id: e.policy_id().to_string(),
                    error: e.to_string(),
                })
                .collect(),
            warnings: result
                .validation_warnings()
                .map(|w| ValidationWarning {
                    policy_id: w.policy_id().to_string(),
                    warning: w.to_string(),
                })
                .collect(),
            msg: result.to_string(),
        }
    }
}

#[pymethods]
impl ValidationResult {
    /// Check if validation passed.
    ///
    /// There may still be non-fatal warnings present when validation passes.
    ///
    /// Returns:
    ///     True if there is no error, else False
    #[getter]
    fn passed(&self) -> bool {
        self.passed
    }

    /// Check if validation passed without warning.
    ///
    /// Returns:
    ///    True if there is no error and no warning, else False
    #[getter]
    fn passed_without_warning(&self) -> bool {
        self.passwed_without_warning
    }

    /// Get errors emitted during validation.
    ///
    /// Returns:
    ///     A list of error messages
    #[getter]
    fn errors(&self) -> Vec<ValidationError> {
        self.errors.clone()
    }

    /// Get warnings emitted during validation.
    ///
    /// Returns:
    ///     A list of warning messages
    #[getter]
    fn warnings(&self) -> Vec<ValidationWarning> {
        self.warnings.clone()
    }

    /// Serialize validation result into python dictionary.
    ///
    /// Returns:
    ///     A python dictionary
    ///
    /// See also:
    ///     * <https://docs.rs/cedar-policy/latest/cedar_policy/struct.ValidationResult.html>
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        pythonize::pythonize(py, &self)
            .or_value_error("failed to serialize validation result to dict")
    }

    /// Return a string representation of the validation result.
    ///
    /// Returns:
    ///     A string which can be used in error messages.
    fn to_string(&self) -> String {
        self.msg.clone()
    }
}

/// Object containing schema information used by the validator.
///
/// A schema is a declaration of the structure of the entity types that you want to
/// support in your application and for which you want Cedar to provide authorization services.
/// After you define a schema, you can ask Cedar to validate your policies against
/// it to ensure that your policies do not contain type errors, such as referencing
/// the entities and their attributes incorrectly.
///
/// See also:
///     * <https://docs.cedarpolicy.com/schema/schema.html>
#[pyclass(module = "cedar._lib")]
#[derive(Clone)]
pub struct Schema {
    fragment: cedar_policy::SchemaFragment,
    validator: cedar_policy::Validator,
    pub schema: cedar_policy::Schema,
}

impl Schema {
    fn from_cedar_fragment(fragment: cedar_policy::SchemaFragment) -> PyResult<Self> {
        let schema = cedar_policy::Schema::from_schema_fragments(iter::once(fragment.clone()))
            // Zip schema with fragment so that we can serialize to string later
            .or_value_error("failed to parse schema from fragment")?;
        let validator = cedar_policy::Validator::new(schema.clone());
        Ok(Schema {
            fragment,
            validator,
            schema,
        })
    }
}

#[pymethods]
impl Schema {
    /// Create a schema from a JSON string.
    ///
    /// Parameters:
    ///     text: a string in JSON format
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/schema/json-schema.html>
    #[staticmethod]
    #[pyo3(signature = (text, /))]
    fn from_json(text: &str) -> PyResult<Schema> {
        cedar_policy::SchemaFragment::from_json_str(&text)
            .or_value_error("failed to parse schema from json")
            .and_then(Schema::from_cedar_fragment)
    }

    /// Create a schema from a cedar language string.
    ///
    /// Parameters:
    ///     text: a string in Cedar format
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/schema/human-readable-schema.html>
    #[staticmethod]
    #[pyo3(signature = (text, /))]
    fn from_string(text: &str) -> PyResult<Schema> {
        cedar_policy::SchemaFragment::from_str(&text)
            .or_value_error("failed to parse schema from string")
            .and_then(Schema::from_cedar_fragment)
    }

    /// Create a schema from a python dictionary.
    ///
    /// Parameters:
    ///     values: dictionary holding schema definition
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/schema/json-schema.html>
    #[staticmethod]
    #[pyo3(signature = (values, /))]
    fn from_dict(values: &Bound<'_, PyDict>) -> PyResult<Self> {
        pythonize::depythonize(values)
            .or_value_error("failed to parse dict")
            .and_then(|value| {
                cedar_policy::SchemaFragment::from_json_value(value)
                    .or_value_error("failed to parse json value")
            })
            .and_then(Schema::from_cedar_fragment)
    }

    /// Serialize schema to python dictionary.
    ///
    /// Returns:
    ///     A python dictionary
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/schema/json-schema.html>
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.fragment
            .clone()
            .to_json_value()
            .or_value_error("failed to serialize fragment to json values")
            .and_then(|values| {
                pythonize::pythonize(py, &values).or_value_error("failed to serialize to dict")
            })
    }

    /// Serialize schema to JSON string.
    ///
    /// Returns:
    ///     A string in JSON format
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/schema/json-schema.html>
    fn to_json(&self) -> PyResult<String> {
        self.fragment
            .to_json_string()
            .or_value_error("failed to encode schema")
    }

    /// Serialize schema to cedar language string.
    ///
    /// Returns:
    ///     A string in Cedar format
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/schema/human-readable-schema.html>
    fn to_string(&self) -> PyResult<String> {
        self.fragment
            .to_cedarschema()
            .or_value_error("failed to encode schema")
    }

    /// Validate given policies against the schema.
    ///
    /// Parameters:
    ///     policies: the policies to validate
    ///
    /// Returns:
    ///     A validation result
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/validation.html>
    #[pyo3(signature = (policies, /))]
    pub fn validate_policies(&self, policies: &PolicySet) -> ValidationResult {
        let result = self
            .validator
            .validate(&policies.policy_set, cedar_policy::ValidationMode::Strict);
        ValidationResult::from_cedar_validation_result(&result)
    }
}
