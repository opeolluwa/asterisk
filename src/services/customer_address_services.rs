use crate::errors::service_error::ServiceError;
use crate::extractors::customer_address::{
    CreateCustomerAddressRequest, UpdateCustomerAddressCommand,
};
use crate::models::customer_address;
use crate::repositories::customer_address_repository::CustomerAddressRepositoryExt;

#[derive(Clone)]
pub struct CustomerAddressService<R: CustomerAddressRepositoryExt> {
    repo: R,
}

impl<R: CustomerAddressRepositoryExt> CustomerAddressService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

pub trait CustomerAddressServiceExt {
    async fn create_customer_address(
        &self,
        req: &CreateCustomerAddressRequest,
        customer_identifier: &str,
    ) -> Result<customer_address::Model, ServiceError>;

    async fn find_customer_address_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<customer_address::Model, ServiceError>;

    async fn find_addresses_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<Vec<customer_address::Model>, ServiceError>;

    async fn find_all_customer_address(&self)
    -> Result<Vec<customer_address::Model>, ServiceError>;

    async fn update_customer_address_by_identifier(
        &self,
        identifier: &str,
        req: &UpdateCustomerAddressCommand,
    ) -> Result<customer_address::Model, ServiceError>;

    async fn delete_customer_address_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<(), ServiceError>;

    async fn count_customer_address(&self) -> Result<i64, ServiceError>;

    async fn count_addresses_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<i64, ServiceError>;
}

impl<R: CustomerAddressRepositoryExt + Send + Sync> CustomerAddressServiceExt
    for CustomerAddressService<R>
{
    async fn create_customer_address(
        &self,
        req: &CreateCustomerAddressRequest,
        customer_identifier: &str,
    ) -> Result<customer_address::Model, ServiceError> {
        Ok(self
            .repo
            .create_customer_address(req, customer_identifier)
            .await?)
    }

    async fn find_customer_address_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<customer_address::Model, ServiceError> {
        Ok(self
            .repo
            .find_customer_address_by_identifier(identifier)
            .await?)
    }

    async fn find_addresses_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<Vec<customer_address::Model>, ServiceError> {
        Ok(self
            .repo
            .find_addresses_by_customer_identifier(customer_identifier)
            .await?)
    }

    async fn find_all_customer_address(
        &self,
    ) -> Result<Vec<customer_address::Model>, ServiceError> {
        Ok(self.repo.find_all_customer_address().await?)
    }

    async fn update_customer_address_by_identifier(
        &self,
        identifier: &str,
        req: &UpdateCustomerAddressCommand,
    ) -> Result<customer_address::Model, ServiceError> {
        Ok(self
            .repo
            .update_customer_address_by_identifier(identifier, req)
            .await?)
    }

    async fn delete_customer_address_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<(), ServiceError> {
        Ok(self
            .repo
            .delete_customer_address_by_identifier(identifier)
            .await?)
    }

    async fn count_customer_address(&self) -> Result<i64, ServiceError> {
        Ok(self.repo.count_customer_address().await?)
    }

    async fn count_addresses_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<i64, ServiceError> {
        Ok(self
            .repo
            .count_addresses_by_customer_identifier(customer_identifier)
            .await?)
    }
}
