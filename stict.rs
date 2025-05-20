impl CombinedValidator {
    pub fn strict(&self) -> Option<bool> {
        match self {
            CombinedValidator::StrictString(v) => Some(v.strict),
            CombinedValidator::StrictInt(v) => Some(v.strict),
            CombinedValidator::StrictFloat(v) => Some(v.strict),
            CombinedValidator::StrictBool(v) => Some(v.strict),
            CombinedValidator::List(v) => v.strict,
            CombinedValidator::Dict(v) => v.strict,
            // Add any other variants that support strictness here...

            _ => None,
        }
    }
}
