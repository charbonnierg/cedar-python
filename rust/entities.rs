use pyo3::prelude::*;
use pyo3::types::PyList;
use std::io::BufWriter;

use crate::entity::*;
use crate::errors::*;
use crate::schema::*;

/// Represents an entity hierarchy, and allows looking up [Entity][cedar.Entity] objects by [Uid][cedar.EntityUid].
///
/// Parameters:
///     entities: a list of entities
///     schema: an optional schema used for entities validation
///
/// See also:
///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#entities>
///     * <https://docs.cedarpolicy.com/policies/syntax-entity.html#entity-overview>
#[pyclass(module = "cedar._lib")]
#[derive(Clone)]
pub struct Entities {
    entities: cedar_policy::Entities,
}

impl Entities {
    pub fn make_cedar_entities(&self, schema: Option<&Schema>) -> PyResult<cedar_policy::Entities> {
        let cedar_schema: Option<&cedar_policy::Schema> = schema.map(|s| &s.schema);
        match cedar_schema {
            Some(schema) => {
                cedar_policy::Entities::from_entities(self.entities.clone(), Some(schema))
                    .or_value_error("failed to validate entities")
            }
            None => Ok(self.entities.clone()),
        }
    }
    pub fn empty() -> Self {
        Entities {
            entities: cedar_policy::Entities::empty(),
        }
    }

    pub fn get_entities(self) -> cedar_policy::Entities {
        self.entities
    }
}

#[pymethods]
impl Entities {
    #[new]
    #[pyo3(signature = (entities, *, schema = None))]
    fn new_py(entities: Vec<Entity>, schema: Option<&Schema>) -> PyResult<Self> {
        let cedar_schema = schema.map(|s| &s.schema);
        cedar_policy::Entities::from_entities(
            entities.iter().map(|e| e.entity.clone()),
            cedar_schema,
        )
        .or_value_error("failed to validate entities")
        .map(|entities| Entities { entities })
    }

    /// Create entities from a JSON string.
    ///
    /// If a schema argument is provided, entities definitions
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
        cedar_policy::Entities::from_json_str(&text, cedar_schema)
            .or_value_error("failed to read json")
            .map(|entities| Entities { entities })
    }

    /// Create entities from a python list of entities as dict.
    ///
    /// If a schema argument is provided, entities definitions
    /// will be validated against given schema.
    ///
    /// Parameters:
    ///     values: list holding entities as dictionaries
    ///     schema: an optional schema used for entity validation
    ///
    /// Returns:
    ///     An entity
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#entities-syntax>
    #[staticmethod]
    #[pyo3(signature = (values, /, *, schema = None))]
    fn from_list(values: &Bound<'_, PyList>, schema: Option<&Schema>) -> PyResult<Self> {
        let values: Vec<serde_json::Value> =
            pythonize::depythonize(values).or_value_error("failed to parse list")?;
        let cedar_schema = schema.map(|s| &s.schema);
        let mut entities: Vec<cedar_policy::Entity> = Vec::new();
        for value in values {
            let entity = cedar_policy::Entity::from_json_value(value, None)
                .or_value_error("failed to parse entity")?;
            entities.push(entity);
        }
        let cedar_entities = cedar_policy::Entities::from_entities(entities, cedar_schema)
            .or_value_error("failed to parse entities")?;
        return Ok(Entities {
            entities: cedar_entities,
        });
    }

    /// Serialize entities to JSON string.
    ///
    /// Returns:
    ///     A string in JSON format
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#entities-syntax>
    fn to_json(&self) -> PyResult<String> {
        let mut buf = BufWriter::new(Vec::new());
        self.entities
            .write_to_json(&mut buf)
            .or_value_error("failed to encode entities")
            .and_then(|_| buf.into_inner().or_value_error("internal error"))
            .and_then(|vec| String::from_utf8(vec).or_value_error("internal error"))
    }

    /// Serialize entities to a python list of dictionaries.
    ///
    /// Returns:
    ///     A python list holding entities as dict
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#entities-syntax>
    fn to_list<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let mut values: Vec<serde_json::Value> = Vec::new();
        for entity in self.entities.iter() {
            let value = entity.to_json_value().or_value_error("internal error")?;
            values.push(value)
        }
        pythonize::pythonize(py, &values).or_value_error("failed to serialize to dict")
    }
}
