use super::*;

pub(super) fn validate(
    values: &AggregateVector,
    attributes: &[ListElement],
) -> Result<bool, GraphError> {
    super::attribute_rules::validate(values.sexptype(), values.len(), attributes)
}
