use crate::errors::app_error::AppError;

#[derive(Clone)]
pub struct RootService {}

impl RootService {
    pub fn init() -> Self {
        Self {}
    }
}
pub trait RootServiceTrait {
    fn health_check(&self) -> Result<(), AppError>;
}

impl RootServiceTrait for RootService {
    fn health_check(&self) -> Result<(), AppError> {
        tracing::info!("application is healthy ...");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_check_returns_ok() {
        let service = RootService::init();
        assert!(service.health_check().is_ok());
    }
}
