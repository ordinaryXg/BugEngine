use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ConfirmationError {
    #[error("destructive action '{0}' requires confirmed: true")]
    NeedsConfirmation(String),
}

pub fn needs_confirmation(args: &serde_json::Value, action: &str) -> Result<(), ConfirmationError> {
    let confirmed = args
        .get("confirmed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if confirmed {
        Ok(())
    } else {
        Err(ConfirmationError::NeedsConfirmation(action.to_string()))
    }
}

impl ConfirmationError {
    pub fn code(&self) -> &'static str {
        match self {
            ConfirmationError::NeedsConfirmation(_) => "NEEDS_CONFIRMATION",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_unconfirmed_destructive_action() {
        assert_eq!(
            needs_confirmation(&json!({}), "delete_node").unwrap_err(),
            ConfirmationError::NeedsConfirmation("delete_node".into())
        );
    }

    #[test]
    fn accepts_confirmed_action() {
        assert!(needs_confirmation(&json!({ "confirmed": true }), "delete_node").is_ok());
    }
}
