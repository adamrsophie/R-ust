use super::*;

// SDK imports accept normalized metadata; setters perform coercion and padding separately.
pub(super) fn validate(
    values: &AtomicVector,
    attributes: &[ListElement],
) -> Result<bool, GraphError> {
    if let AtomicVector::String(strings) = values {
        if strings.iter().flatten().any(|text| text.contains('\0')) {
            return Err(GraphError::InvalidCharacter);
        }
    }
    super::attribute_rules::validate(values.sexptype(), values.len(), attributes)
}
