//! Form structures and validation for hoofprint web application

use serde::Deserialize;
use uuid::Uuid;

use crate::error::HoofprintError;

#[derive(Debug, Deserialize)]
pub struct CreateCodeForm {
    pub code_type: String,
    pub code_value: String,
    pub site_id: String,
    pub code_name: Option<String>,
}

impl CreateCodeForm {
    /// Validate the form data
    pub fn validate(&self) -> Result<(), HoofprintError> {
        let mut errors = Vec::new();

        // Validate code_type
        if self.code_type != "barcode" && self.code_type != "qrcode" {
            errors.push("Code type must be either 'barcode' or 'qrcode'".to_string());
        }

        // Validate code_value
        if self.code_value.is_empty() {
            errors.push("Code value cannot be empty".to_string());
        } else if self.code_value.len() > 255 {
            errors.push("Code value must be 255 characters or less".to_string());
        }

        // Validate site_id is a valid UUID
        if Uuid::parse_str(&self.site_id).is_err() {
            errors.push("Site ID must be a valid UUID".to_string());
        }

        // Validate code_name if provided
        if let Some(ref name) = self.code_name
            && name.len() > 255
        {
            errors.push("Code name must be 255 characters or less".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(HoofprintError::ValidationError(errors))
        }
    }

    /// Parse the site_id as a UUID after validation
    pub fn parse_site_id(&self) -> Result<Uuid, HoofprintError> {
        Uuid::parse_str(&self.site_id)
            .map_err(|_| HoofprintError::ValidationError(vec!["Invalid site ID".to_string()]))
    }
}

#[derive(Debug, Deserialize)]
pub struct EditCodeForm {
    pub code_type: String,
    pub code_value: String,
    pub site_id: String,
    pub code_name: Option<String>,
}

impl EditCodeForm {
    /// Validate the form data (same rules as CreateCodeForm)
    pub fn validate(&self) -> Result<(), HoofprintError> {
        let mut errors = Vec::new();

        // Validate code_type
        if self.code_type != "barcode" && self.code_type != "qrcode" {
            errors.push("Code type must be either 'barcode' or 'qrcode'".to_string());
        }

        // Validate code_value
        if self.code_value.is_empty() {
            errors.push("Code value cannot be empty".to_string());
        } else if self.code_value.len() > 255 {
            errors.push("Code value must be 255 characters or less".to_string());
        }

        // Validate site_id is a valid UUID
        if Uuid::parse_str(&self.site_id).is_err() {
            errors.push("Site ID must be a valid UUID".to_string());
        }

        // Validate code_name if provided
        if let Some(ref name) = self.code_name
            && name.len() > 255
        {
            errors.push("Code name must be 255 characters or less".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(HoofprintError::ValidationError(errors))
        }
    }

    /// Parse the site_id as a UUID after validation
    pub fn parse_site_id(&self) -> Result<Uuid, HoofprintError> {
        Uuid::parse_str(&self.site_id)
            .map_err(|_| HoofprintError::ValidationError(vec!["Invalid site ID".to_string()]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_code_type() {
        let form = CreateCodeForm {
            code_type: "barcode".to_string(),
            code_value: "123456".to_string(),
            site_id: "00000000-0000-0000-0000-000000000000".to_string(),
            code_name: None,
        };
        assert!(form.validate().is_ok());

        let form = CreateCodeForm {
            code_type: "qrcode".to_string(),
            code_value: "123456".to_string(),
            site_id: "00000000-0000-0000-0000-000000000000".to_string(),
            code_name: None,
        };
        assert!(form.validate().is_ok());

        let form = CreateCodeForm {
            code_type: "invalid".to_string(),
            code_value: "123456".to_string(),
            site_id: "00000000-0000-0000-0000-000000000000".to_string(),
            code_name: None,
        };
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_validate_code_value() {
        let form = CreateCodeForm {
            code_type: "barcode".to_string(),
            code_value: "".to_string(),
            site_id: "00000000-0000-0000-0000-000000000000".to_string(),
            code_name: None,
        };
        assert!(form.validate().is_err());

        let form = CreateCodeForm {
            code_type: "barcode".to_string(),
            code_value: "a".repeat(256),
            site_id: "00000000-0000-0000-0000-000000000000".to_string(),
            code_name: None,
        };
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_validate_site_id() {
        let form = CreateCodeForm {
            code_type: "barcode".to_string(),
            code_value: "123456".to_string(),
            site_id: "not-a-uuid".to_string(),
            code_name: None,
        };
        assert!(form.validate().is_err());
    }
}
