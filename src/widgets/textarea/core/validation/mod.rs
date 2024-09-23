pub mod validators;

use {super::TextArea, std::sync::Arc};

pub enum ValidationResult {
    Valid,
    Invalid(Vec<String>),
}

type ValidatorFnType = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

#[derive(Clone)]
pub struct ValidatorFn(ValidatorFnType);

impl ValidatorFn {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        ValidatorFn(Arc::new(f))
    }

    // Method to call the inner function
    pub fn call(&self, arg: &str) -> Result<(), String> {
        (self.0)(arg)
    }
}

impl std::fmt::Debug for ValidatorFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CloneableFn {{ ... }}")
    }
}

impl<'a> TextArea<'a> {
    pub fn validate(&self) -> ValidationResult {
        let lines = self.lines().join("\n");
        let mut errors = Vec::new();

        // For each validation function, call it and collect the errors
        for validation in &self.validators {
            match validation.call(&lines) {
                Ok(_) => {}
                Err(err) => errors.push(err),
            }
        }

        if errors.is_empty() {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid(errors)
        }
    }

    pub fn is_valid(&self) -> bool {
        matches!(self.validate(), ValidationResult::Valid)
    }
}
