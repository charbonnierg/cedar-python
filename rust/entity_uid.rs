use crate::errors::IntoPyErr;
use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Entity Unique ID
///
/// An entity represents a principal, action, or resource in your authorization model.
/// Each entity has a unique ID composed of a type name and a resource id.
///
/// See also:
///     * <https://docs.cedarpolicy.com/policies/syntax-datatypes.html#datatype-entity>
#[pyclass(eq, frozen, hash, module = "cedar._lib")]
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct EntityUid {
    #[serde(rename = "type")]
    entity_type: String,
    #[serde(rename = "id")]
    entity_id: String,
}

impl EntityUid {
    /// Create a new EntityUid from a cedar_policy::EntityUid.
    ///
    /// This function is not exposed to python as there is no way
    /// to access a cedar_policy::EntityUid on python side.
    pub fn from_cedar_entity_uid(euid: &cedar_policy::EntityUid) -> Self {
        EntityUid {
            entity_type: euid.type_name().to_string(),
            entity_id: euid.id().escaped().to_string(),
        }
    }

    /// Create a new cedar_policy::EntityUid out of EntityUid
    ///
    /// This function is not exposed to python as there is no way
    /// to access a cedar_policy::EntityUid on python side.
    pub fn make_cedar_euid(&self) -> PyResult<cedar_policy::EntityUid> {
        let eid = cedar_policy::EntityId::from_str(&self.entity_id)
            .unwrap_or_else(|never| match never {});
        let etn = cedar_policy::EntityTypeName::from_str(&self.entity_type)
            .or_value_error("internal error")?;
        Ok(cedar_policy::EntityUid::from_type_name_and_id(etn, eid))
    }
}

#[pymethods]
impl EntityUid {
    /// Create an EntityUid from JSON string.
    ///
    /// Parameters:
    ///     text: a string in JSON format
    ///
    /// Returns:
    ///     An entity uid.
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#uid>
    #[staticmethod]
    #[pyo3(signature = (text, /))]
    fn from_json(text: &str) -> PyResult<Self> {
        serde_json::from_str(&text)
            .or_value_error("failed to parse json")
            .and_then(|value| {
                cedar_policy::EntityUid::from_json(value)
                    .map(|euid| Self::from_cedar_entity_uid(&euid))
                    .or_value_error("failed to parse entity uid")
            })
    }

    /// Create an EntityUid from a python dict.
    ///
    /// Parameters:
    ///     values: a dict holding uid definition.
    ///
    /// Returns:
    ///     An entity uid.
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#uid>
    #[staticmethod]
    fn from_dict(values: &Bound<'_, PyDict>) -> PyResult<Self> {
        pythonize::depythonize(values)
            .or_value_error("failed to parse dict")
            .and_then(|json| serde_json::from_value(json).or_value_error("failed to parse dict"))
    }

    /// Create an EntityUid from a string.
    ///
    /// Parameters:
    ///     text: a string
    ///
    /// Returns:
    ///     An entity uid.
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/syntax-datatypes.html#datatype-entity>
    #[staticmethod]
    #[pyo3(signature = (text, /))]
    fn from_string(text: &str) -> PyResult<Self> {
        cedar_policy::EntityUid::from_str(&text)
            .map(|e| Self::from_cedar_entity_uid(&e))
            .or_value_error("failed to parse entity uid")
    }

    /// Create a new Euid from entity type name and id
    ///
    /// Parameters:
    ///     name: the entity type name
    ///     id: the entity id
    ///
    /// Returns:
    ///     An entity uid
    #[staticmethod]
    fn from_type_name_and_id(name: &str, id: &str) -> PyResult<Self> {
        // Parsing entity id never fails
        let eid = cedar_policy::EntityId::from_str(&id).unwrap_or_else(|never| match never {});
        cedar_policy::EntityTypeName::from_str(&name)
            .map(|etn| cedar_policy::EntityUid::from_type_name_and_id(etn, eid))
            .map(|euid| Self::from_cedar_entity_uid(&euid))
            .or_value_error("failed to parse entity uid")
    }

    /// Serialize entity uid to python dictionary.
    ///
    /// Returns:
    ///     A python dictionary with two keys: `"type"` and `"id"`
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#uid>
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        pythonize::pythonize(py, self).or_value_error("failed to serialize to dict")
    }

    /// Serialize entity uid to JSON string.
    ///
    /// Returns:
    ///     A string in JSON format
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/auth/entities-syntax.html#uid>
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self).or_value_error("failed to encode entity uid")
    }

    /// Serialize entity uid to string.
    ///
    /// Returns:
    ///     A string
    ///
    /// See also:
    ///     * <https://docs.cedarpolicy.com/policies/syntax-datatypes.html#datatype-entity>
    fn to_string(&self) -> PyResult<String> {
        self.make_cedar_euid().map(|euid| euid.to_string())
    }

    /// Get entity type name
    ///
    /// Returns:
    ///     The type name of the entity
    #[getter]
    fn entity_type(&self) -> String {
        self.entity_type.clone()
    }

    /// Get entity id
    ///
    /// Returns:
    ///     The id of the entity
    #[getter]
    fn entity_id(&self) -> String {
        self.entity_id.clone()
    }

    fn __str__(&self) -> PyResult<String> {
        self.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "EntityUid(entity_type=\"{entity_type}\", entity_id=\"{entity_id}\")",
            entity_type = self.entity_type,
            entity_id = self.entity_id
        )
    }
}
