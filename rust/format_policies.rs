use pyo3::prelude::*;

use crate::errors::IntoPyErr;

/// Format given policy.
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
#[pyfunction]
#[pyo3(signature = (text, /, *, line_width = None, indent_width = None))]
pub fn format_policies(
    text: &str,
    line_width: Option<usize>,
    indent_width: Option<isize>,
) -> PyResult<String> {
    cedar_policy_formatter::policies_str_to_pretty(
        &text,
        &cedar_policy_formatter::Config {
            line_width: line_width.unwrap_or(88),
            indent_width: indent_width.unwrap_or(2),
        },
    )
    .or_value_error("failed to format policies")
}
