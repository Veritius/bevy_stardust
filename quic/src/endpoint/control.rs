use crate::EndpointShared;

pub struct EndpointControl<'a> {
    endpoint: &'a mut EndpointShared,
}