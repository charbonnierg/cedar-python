use pyo3::prelude::*;

use crate::entities::*;
use crate::errors::*;
use crate::policy_set::*;
use crate::request::*;
use crate::response::*;
use crate::schema::*;

/// Cedar authorizer
///
/// If a schema is provided, policies are validated against the schema.
///
/// Schema is also used to validate entities when using
/// [Authorizer.is_authorized][cedar.Authorizer.is_authorized] and
/// [Authorizer.is_authorized_batch][cedar.Authorizer.is_authorized_batch]
///
/// Parameters:
///     policies: the policies to used when checking authorization
///     schema: the schema used to verify policies, entities and requests
#[pyclass(module = "cedar._lib")]
pub struct Authorizer {
    policies: PolicySet,
    schema: Option<Schema>,
    authorizer: cedar_policy::Authorizer,
}

impl Authorizer {
    fn new(
        policies: Option<PolicySet>,
        schema: Option<Schema>,
    ) -> Result<Authorizer, ValidationResult> {
        let authorizer = cedar_policy::Authorizer::new();
        let policies = policies.unwrap_or(PolicySet {
            policy_set: cedar_policy::PolicySet::new(),
        });
        match schema.as_ref() {
            Some(schema) => {
                let result = schema.validate_policies(&policies);
                if !result.passed {
                    return Err(result);
                }
            }
            None => (),
        }
        Ok(Authorizer {
            policies,
            schema,
            authorizer,
        })
    }
}

#[pymethods]
impl Authorizer {
    #[new]
    #[pyo3(signature = (policies = None, schema = None))]
    fn new_py(policies: Option<PolicySet>, schema: Option<Schema>) -> PyResult<Authorizer> {
        Self::new(policies, schema).or_else(|result| {
            let errors: Vec<String> = result
                .errors
                .into_iter()
                .map(|e| e.error.to_string())
                .collect();
            let msg = errors.join(" - ");
            Err(&msg).or_value_error("invalid policies")
        })
    }

    /// Check if principal is authorized to perform action on resource within context.
    ///
    /// Parameters:
    ///     request: a request describing principal, action, resource and context
    ///     entities: the entities to consider when applying policies
    ///
    /// Returns:
    ///     An authorization response
    #[pyo3(signature = (request, entities = None))]
    fn is_authorized(&self, request: &Request, entities: Option<&Entities>) -> PyResult<Response> {
        let schema = self.schema.as_ref();
        let entities = entities
            .unwrap_or(&Entities::empty())
            .make_cedar_entities(schema)?;
        let cedar_request = request.make_cedar_request(schema)?;
        let response =
            self.authorizer
                .is_authorized(&cedar_request, &self.policies.policy_set, &entities);
        Ok(Response::from_cedar_response(
            response,
            request.correlation_id.clone(),
        ))
    }

    /// Check if list of requests are authorized.
    ///
    /// Parameters:
    ///     requests: a list of requests describing principals, actions, resources and contexts
    ///     entities: the entities to consider when applying policies
    ///
    /// Returns:
    ///     A list of authorization responses
    #[pyo3(signature = (requests, entities = None))]
    fn is_authorized_batch(
        &self,
        requests: Vec<Request>,
        entities: Option<&Entities>,
    ) -> PyResult<Vec<Response>> {
        let schema = self.schema.as_ref();
        let entities = entities
            .unwrap_or(&Entities::empty())
            .make_cedar_entities(schema)?;
        let mut responses: Vec<Response> = Vec::new();
        for request in requests {
            let cedar_request = request.make_cedar_request(schema)?;
            let response =
                self.authorizer
                    .is_authorized(&cedar_request, &self.policies.policy_set, &entities);
            responses.push(Response::from_cedar_response(
                response,
                request.correlation_id.clone(),
            ));
        }
        Ok(responses)
    }
}

/// Check if principal is authorized to perform action on resource within context.
///
/// Parameters:
///     request: a request describing principal, action, resource and context
///     policies: the policies to apply when checking authorization
///     entities: the entities to consider when applying policies
///
/// Returns:
///     A list of authorization responses
#[pyfunction]
#[pyo3(signature = (request, policies, entities = None, schema = None))]
pub fn is_authorized(
    request: &Request,
    policies: &PolicySet,
    entities: Option<&Entities>,
    schema: Option<&Schema>,
) -> PyResult<Response> {
    let authorizer = cedar_policy::Authorizer::new();
    let entities = entities
        .unwrap_or(&Entities::empty())
        .make_cedar_entities(schema)?;
    let policy_set = &policies.policy_set;
    let cedar_request = request.make_cedar_request(schema)?;
    let response = authorizer.is_authorized(&cedar_request, policy_set, &entities);
    Ok(Response::from_cedar_response(
        response,
        request.correlation_id.clone(),
    ))
}

/// Check if list of requests are authorized.
///
/// Parameters:
///     requests: a list of authorization requests
///     policies: the policies to apply when checking authorization
///     entities: the entities to consider when applying policies
///
/// Returns:
///     A list of authorization responses
#[pyfunction]
#[pyo3(signature = (requests, policies, entities = None, schema = None))]
pub fn is_authorized_batch(
    requests: Vec<Request>,
    policies: PolicySet,
    entities: Option<&Entities>,
    schema: Option<&Schema>,
) -> PyResult<Vec<Response>> {
    let policy_set = &policies.policy_set;
    let authorizer = cedar_policy::Authorizer::new();
    let entities = entities
        .unwrap_or(&Entities::empty())
        .make_cedar_entities(schema)?;
    let mut responses: Vec<Response> = Vec::new();
    for request in requests {
        let cedar_request = request.make_cedar_request(schema)?;
        let response = authorizer.is_authorized(&cedar_request, policy_set, &entities);
        responses.push(Response::from_cedar_response(
            response,
            request.correlation_id.clone(),
        ));
    }
    Ok(responses)
}
