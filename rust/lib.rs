use pyo3::prelude::*;

mod authorizer;
mod entities;
mod entity;
mod entity_uid;
mod errors;
mod format_policies;
mod policy;
mod policy_set;
mod request;
mod response;
mod schema;
use crate::authorizer::*;
use crate::entities::*;
use crate::entity::*;
use crate::entity_uid::*;
use crate::policy::*;
use crate::policy_set::*;
use crate::request::*;
use crate::response::*;
use crate::schema::*;

/// A Python module implemented in Rust.
#[pymodule(name = " _lib", module = "cedar")]
fn setup_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<EntityUid>()?;
    m.add_class::<Effect>()?;
    m.add_class::<Entity>()?;
    m.add_class::<Entities>()?;
    m.add_class::<PolicySet>()?;
    m.add_class::<Policy>()?;
    m.add_class::<ValidationResult>()?;
    m.add_class::<Schema>()?;
    m.add_class::<Authorizer>()?;
    m.add_class::<Request>()?;
    m.add_class::<Decision>()?;
    m.add_class::<Diagnostics>()?;
    m.add_class::<Response>()?;
    m.add_function(wrap_pyfunction!(is_authorized, m)?)?;
    m.add_function(wrap_pyfunction!(is_authorized_batch, m)?)?;
    m.add_function(wrap_pyfunction!(format_policies::format_policies, m)?)?;
    Ok(())
}
