1. Basic Strictness Override in Union
rust
#[test]
fn test_union_with_strict_false_override() {
    Python::with_gil(|py| {
        // Setup: A union with one strict and one non-strict field
        let strict_int = Validator::Int(IntValidator { strict: true });
        let non_strict_int = Validator::Int(IntValidator { strict: false });

        let union_validator = UnionValidator {
            choices: vec![
                (strict_int, None),  // Strict validator
                (non_strict_int, None),  // Non-strict validator
            ],
            strict: true,  // Union itself is strict
            custom_error: None,
        };

        let mut state = ValidationState::new(true);  // Global strict=true

        // Test: String input ("123") should work for non-strict validator
        let input = "123";
        let result = union_validator.validate_smart(py, &input, &mut state);
        assert!(result.is_ok());  // Should pass due to non-strict path
    });
}
2. Nested Unions with Mixed Strictness
rust
#[test]
fn test_nested_unions_with_strictness() {
    Python::with_gil(|py| {
        // Inner Union: One strict, one non-strict
        let inner_union = UnionValidator {
            choices: vec![
                (create_field_validator(false), None),  // Non-strict
                (create_field_validator(true), None),   // Strict
            ],
            strict: true,
            custom_error: None,
        };

        // Outer Union: Contains inner union + a strict validator
        let outer_union = UnionValidator {
            choices: vec![
                (Validator::Union(inner_union), None),  // Inner union (mixed)
                (create_field_validator(true), None),   // Strict
            ],
            strict: false,  // Outer union is non-strict
            custom_error: None,
        };

        let mut state = ValidationState::new(true);  // Global strict=true
        let input = "123";

        // Should pass because inner union has a non-strict path
        let result = outer_union.validate_smart(py, &input, &mut state);
        assert!(result.is_ok());
    });
}
3. Field-Level strict=False Overrides Model-Level Strict
rust
#[test]
fn test_field_strict_false_overrides_model_strict() {
    Python::with_gil(|py| {
        // Model is strict, but field has `strict=False`
        let model_validator = ModelValidator {
            fields: vec![Field {
                name: "test_field",
                validator: Arc::new(Validator::Int(IntValidator { strict: false })),
                strict: Some(false),  // Explicit override
                frozen: false,
            }],
            strict: true,  // Model-level strict
            // ... other fields ...
        };

        let mut state = ValidationState::new(true);  // Global strict=true
        let input = "123";

        // Should pass because field overrides strictness
        let result = model_validator.validate(py, &input, &mut state);
        assert!(result.is_ok());
    });
}
4. Union with Annotated[Type, Field(strict=False)]
rust
#[test]
fn test_annotated_with_strict_false_in_union() {
    Python::with_gil(|py| {
        // Simulate `Annotated[Int, Field(strict=False)] | Annotated[Str, Field(strict=True)]`
        let non_strict_int = Validator::FieldValidator(FieldValidator {
            validator: Arc::new(Validator::Int(IntValidator { strict: false })),
            strict: Some(false),  // Explicitly non-strict
            // ... other fields ...
        });

        let strict_str = Validator::FieldValidator(FieldValidator {
            validator: Arc::new(Validator::Str(StrValidator { strict: true })),
            strict: Some(true),  // Explicitly strict
            // ... other fields ...
        });

        let union_validator = UnionValidator {
            choices: vec![
                (non_strict_int, None),
                (strict_str, None),
            ],
            strict: true,  // Union itself is strict
            custom_error: None,
        };

        let mut state = ValidationState::new(true);  // Global strict=true

        // Test with int-as-string (should pass via non-strict path)
        let input = "123";
        let result = union_validator.validate_smart(py, &input, &mut state);
        assert!(result.is_ok());

        // Test with invalid string (should fail)
        let input = "not_an_int";
        let result = union_validator.validate_smart(py, &input, &mut state);
        assert!(result.is_err());
    });
}
