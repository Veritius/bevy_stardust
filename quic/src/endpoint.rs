use std::sync::Arc;

/// A clonable handle to an endpoint.
#[derive(Clone)]
pub struct Endpoint(Arc<EndpointInner>);

struct EndpointInner {

}