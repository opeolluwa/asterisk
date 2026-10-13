use sanitizer::prelude::*;
use ts_rs::TS;
use validator::Validate;

use crate::models::sea_orm_active_enums::RolePermissionAction;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, message = "name is required"))]
    #[sanitizer(trim, lower_case)]
    pub name: String,
    #[sanitizer(trim)]
    pub description: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateRoleRequest {
    #[sanitizer(trim, lower_case)]
    pub name: Option<String>,
    #[sanitizer(trim)]
    pub description: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AssignRoleRequest {
    #[validate(length(min = 1, message = "user identifier is required"))]
    pub user_identifier: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AddPermissionRequest {
    #[validate(length(min = 1, message = "resource is required"))]
    pub resource: String,
    pub action: RolePermissionAction,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RemovePermissionRequest {
    #[validate(length(min = 1, message = "resource is required"))]
    pub resource: String,
    pub action: RolePermissionAction,
}
