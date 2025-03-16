use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::entity_uid::*;
use crate::errors::*;
use crate::schema::Schema;

/// Entity datatype
///
/// An entity in Cedar is a stored object that serves as the representation for principals, actions, and resources that are part of your application.
///
/// Parameters:
///     euid: the entity unique id
///     parents: a list holding entity unique id of parent entities
///     attrs: a dictionary holding entity attributes
///     schema: an optional schema used for entity validation
///
/// See also:
///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#entities>
///     * <https://docs.cedarpolicy.com/policies/syntax-entity.html#entity-overview>
#[pyclass(module = "cedar._lib")]
#[derive(Clone)]
pub struct Entity {
    pub entity: cedar_policy::Entity,
}

#[pymethods]
impl Entity {
    #[new]
    #[pyo3(signature = (euid, parents, attrs, schema = None))]
    fn new(
        euid: EntityUid,
        parents: Vec<EntityUid>,
        attrs: &Bound<'_, PyDict>,
        schema: Option<&Schema>,
    ) -> PyResult<Self> {
        let cedar_schema = schema.map(|s| &s.schema);
        let attrs: serde_json::Value =
            pythonize::depythonize(attrs).or_value_error("failed to parse attrs")?;
        let parents: serde_json::Value =
            serde_json::to_value(parents).or_value_error("failed to parse parents")?;
        let values = serde_json::json!({
            "uid": euid,
            "parents": parents,
            "attrs": attrs,
        });
        cedar_policy::Entity::from_json_value(values, cedar_schema)
            .or_value_error("failed to parse entity from JSON")
            .map(|entity| Entity { entity })
    }

    /// Create an entity from a python dictionary.
    ///
    /// If a schema argument is provided, entity definition
    /// will be validated against given schema.
    ///
    /// Parameters:
    ///     values: dictionary holding entity definition
    ///     schema: an optional schema used for entity validation
    ///
    /// Returns:
    ///     An entity
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#entities-syntax>
    #[staticmethod]
    #[pyo3(signature = (values, /, *, schema = None))]
    fn from_dict(values: &Bound<'_, PyDict>, schema: Option<&Schema>) -> PyResult<Self> {
        let cedar_schema = schema.map(|s| &s.schema);
        pythonize::depythonize(values)
            .or_value_error("failed to parse dict")
            .and_then(|value| {
                cedar_policy::Entity::from_json_value(value, cedar_schema)
                    .or_value_error("failed to parse json value")
            })
            .map(|entity| Entity { entity })
    }

    /// Create an entity from a JSON string.
    ///
    /// If a schema argument is provided, entity definition
    /// will be validated against given schema.
    ///
    /// Parameters:
    ///     text: a string in JSON format
    ///     schema: an optional schema used for entity validation
    ///
    /// Returns:
    ///     An entity
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#entities-syntax>
    #[staticmethod]
    #[pyo3(signature = (text, /, *, schema = None))]
    fn from_json(text: String, schema: Option<&Schema>) -> PyResult<Self> {
        let cedar_schema = schema.map(|s| &s.schema);
        cedar_policy::Entity::from_json_str(text, cedar_schema)
            .or_value_error("failed to parse entity from JSON")
            .map(|entity| Entity { entity })
    }

    /// Serialize entity to JSON string.
    ///
    /// Returns:
    ///     A string in JSON format
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#entities-syntax>
    fn to_json(&self) -> PyResult<String> {
        self.entity
            .to_json_string()
            .or_value_error("failed to encode entity to JSON")
    }

    /// Serialize entity to a python dict.
    ///
    /// Returns:
    ///     A python dict holding entity definitions
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#entities-syntax>
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.entity
            .clone()
            .to_json_value()
            .or_value_error("failed to serialize entity to JSON")
            .and_then(|values| {
                pythonize::pythonize(py, &values)
                    .or_value_error("failed to serialize entity to dict")
            })
    }
}
