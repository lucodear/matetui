pub fn required_validator(input: &str) -> Result<(), String> {
    if input.is_empty() {
        Err("This field is required".to_string())
    } else {
        Ok(())
    }
}
