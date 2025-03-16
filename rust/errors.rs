use pyo3::prelude::*;

/// A trait to convert Result<T, E> and Option<T> into Result<T, PyErr>
pub trait IntoPyErr<T> {
    fn or_value_error(self, msg: &str) -> Result<T, PyErr>;
}

impl<T> IntoPyErr<T> for Option<T> {
    /// Transform Option<T> into Result<T, PyErr>.
    ///
    /// Provided message will be used as error message.
    ///
    /// Example:
    ///
    /// ```
    /// let err = None.or_value_error("something went wrong")
    /// ```
    ///
    /// would raise `ValueError("something went wrong")`` on python side.
    fn or_value_error(self, msg: &str) -> Result<T, PyErr> {
        match self {
            Some(v) => Ok(v),
            None => Err(pyo3::exceptions::PyValueError::new_err(msg.to_owned())),
        }
    }
}

impl<T, E: ToString> IntoPyErr<T> for Result<T, E> {
    /// Transform Result<T, E> into Result<T, PyErr>.
    ///
    /// Provided message will be used as prefix before
    /// string representation of the error.
    ///
    /// Example:
    ///
    /// ```
    /// let err = Err("BOOM").or_value_error("something went wrong")
    /// ```
    ///
    /// would raise `ValueError("BOOM: something went wrong")`` on python side.
    fn or_value_error(self, msg: &str) -> Result<T, PyErr> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(
                msg.to_owned() + ": " + &e.to_string(),
            )),
        }
    }
}
