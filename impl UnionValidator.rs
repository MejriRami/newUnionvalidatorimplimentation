use pyo3::prelude::*;
use pyo3::types::PyAny;
use crate::validators::Validator;
use crate::validation_state::{Exactness, ValidationState};
use crate::errors::{ValError, ValResult};
use crate::input::Input;
use std::sync::Arc;

/// Extension trait to check for FieldValidator
pub trait ValidatorExt {
    fn as_field_validator(&self) -> Option<&FieldValidator>;
}

impl ValidatorExt for Validator {
    fn as_field_validator(&self) -> Option<&FieldValidator> {
        match self {
            Validator::FieldValidator(fv) => Some(fv),
            _ => None,
        }
    }
}

/// Modified FieldValidator to track strictness override
#[derive(Debug, Clone)]
pub struct FieldValidator {
    pub validator: Arc<Validator>,
    pub strict: Option<bool>, // None = inherit, Some(true/false) = override
    // ... other existing fields ...
}

#[derive(Debug, Clone)]
pub struct UnionValidator {
    pub choices: Vec<(Validator, Option<String>)>,
    pub strict: bool,
    pub custom_error: Option<String>,
    // ... other existing fields ...
}

impl UnionValidator {
    /// Helper to get effective strict mode
    fn effective_strict(&self, state: &ValidationState, validator: &Validator) -> bool {
        if let Some(fv) = validator.as_field_validator() {
            if let Some(strict) = fv.strict {
                return strict;
            }
        }
        self.strict || state.strict
    }

    pub fn validate_smart<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let old_exactness = state.exactness;
        let old_fields_set_count = state.fields_set_count;
        let old_strict = state.strict;

        let mut errors = MaybeErrors::new(self.custom_error.as_ref());
        let mut best_match: Option<(Py<PyAny>, Exactness, Option<usize>)> = None;

        for (choice, label) in &self.choices {
            state.exactness = Some(Exactness::Exact);
            state.fields_set_count = None;
            
            // Apply field-level strict override if present
            state.strict = self.effective_strict(state, choice);

            let result = choice.validate(py, input, state);
            
            // Restore original strict mode
            state.strict = old_strict;

            match result {
                Ok(new_success) => {
                    match (state.exactness, state.fields_set_count) {
                        (Some(Exactness::Exact), None) => {
                            state.exactness = old_exactness;
                            state.fields_set_count = old_fields_set_count;
                            return Ok(new_success);
                        }
                        _ => {
                            let new_exactness = state.exactness.unwrap_or(Exactness::Lax);
                            let new_fields_set_count = state.fields_set_count;

                            let new_success_is_best_match = best_match.as_ref().map_or(true, |(_, cur_exactness, cur_fields_set_count)| {
                                match (*cur_fields_set_count, new_fields_set_count) {
                                    (Some(cur), Some(new)) if cur != new => cur < new,
                                    _ => *cur_exactness < new_exactness,
                                }
                            });

                            if new_success_is_best_match {
                                best_match = Some((new_success, new_exactness, new_fields_set_count));
                            }
                        }
                    }
                },
                Err(ValError::LineErrors(lines)) => {
                    if best_match.is_none() {
                        errors.push(choice, label.as_deref(), lines);
                    }
                }
                otherwise => return otherwise,
            }
        }

        state.exactness = old_exactness;
        state.fields_set_count = old_fields_set_count;

        if let Some((best_match, exactness, fields_set_count)) = best_match {
            state.floor_exactness(exactness);
            if let Some(count) = fields_set_count {
                state.add_fields_set(count);
            }
            return Ok(best_match);
        }

        Err(errors.into_val_error(input))
    }

    pub fn validate_left_to_right<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let old_strict = state.strict;
        let mut errors = MaybeErrors::new(self.custom_error.as_ref());

        for (validator, label) in &self.choices {
            // Apply field-level strict override if present
            state.strict = self.effective_strict(state, validator);

            match validator.validate(py, input, state) {
                Err(ValError::LineErrors(lines)) => {
                    errors.push(validator, label.as_deref(), lines);
                }
                otherwise => {
                    state.strict = old_strict;
                    return otherwise;
                },
            }
            state.strict = old_strict;
        }

        Err(errors.into_val_error(input))
    }
}
