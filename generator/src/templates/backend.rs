use convert_case::{Case, Casing};

#[derive(Debug, Clone)]
pub struct ModuleConfig {
    pub name: String,
    pub name_pascal: String,
    pub description: String,
}

impl ModuleConfig {
    pub fn new(name: &str, description: &str) -> Self {
        let name = name.to_case(Case::Snake);
        let name_pascal = name.to_case(Case::Pascal);

        Self {
            name,
            name_pascal,
            description: description.to_string(),
        }
    }
}

pub fn repository(cfg: &ModuleConfig) -> String {
    format!(
        r#"use sea_orm::{{
    ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, Set,
}};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::extractors::{name}::{{Create{Name}Request, Update{Name}Command}};
use crate::models::{model};
use crate::repositories::base::Repository;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait  {Name}RepositoryExt {{
    async fn create_{name}(
        &self,
        req: &Create{Name}Request,
    ) -> Result<{model}::Model, DatabaseError>;

    async fn find_{name}_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<{model}::Model, DatabaseError>;

    async fn find_all_{name}(&self) -> Result<Vec<{model}::Model>, DatabaseError>;

    async fn update_{name}_by_identifier(
        &self,
        identifier: &str,
        req: &Update{Name}Command,
    ) -> Result<{model}::Model, DatabaseError>;

    async fn delete_{name}_by_identifier(&self, identifier: &str) -> Result<(), DatabaseError>;

    async fn count_{name}(&self) -> Result<i64, DatabaseError>;
}}

#[derive(Debug, Clone)]
pub struct {Name}Repository {{
    db: DatabaseConnection,
}}

impl Repository for {Name}Repository {{
    fn init(db: &DatabaseConnection) -> Self {{
        Self {{ db: db.clone() }}
    }}
}}

impl {Name}RepositoryExt for {Name}Repository {{
    async fn create_{name}(
        &self,
        _req: &Create{Name}Request,
    ) -> Result<{model}::Model, DatabaseError> {{
        todo!()
    }}

    async fn find_{name}_by_identifier(
        &self,
        _identifier: &str,
    ) -> Result<{model}::Model, DatabaseError> {{
        todo!()
    }}

    async fn find_all_{name}(&self) -> Result<Vec<{model}::Model>, DatabaseError> {{
        todo!()
    }}

    async fn update_{name}_by_identifier(
        &self,
        _identifier: &str,
        _req: &Update{Name}Command,
    ) -> Result<{model}::Model, DatabaseError> {{
        todo!()
    }}

    async fn delete_{name}_by_identifier(&self, _identifier: &str) -> Result<(), DatabaseError> {{
        todo!()
    }}

    async fn count_{name}(&self) -> Result<i64, DatabaseError> {{
        todo!()
    }}
}}
"#,
        name = cfg.name,
        model = cfg.name,
        Name = cfg.name_pascal,
    )
}

pub fn service(cfg: &ModuleConfig) -> String {
    format!(
        r#"use crate::errors::service_error::ServiceError;
use crate::extractors::{name}::{{Create{Name}Request, Update{Name}Command}};
use crate::models::{model};
use crate::repositories::{name}_repository::{Name}RepositoryExt;

#[derive(Clone)]
pub struct {Name}Service<R: {Name}RepositoryExt> {{
    repo: R,
}}

impl<R: {Name}RepositoryExt> {Name}Service<R> {{
    pub fn new(repo: R) -> Self {{
        Self {{ repo }}
    }}
}}

pub trait  {Name}ServiceExt {{
    async fn create_{name}(
        &self,
        req: &Create{Name}Request,
    ) -> Result<{model}::Model, ServiceError>;

    async fn find_{name}_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<{model}::Model, ServiceError>;

    async fn find_all_{name}(&self) -> Result<Vec<{model}::Model>, ServiceError>;

    async fn update_{name}_by_identifier(
        &self,
        identifier: &str,
        req: &Update{Name}Command,
    ) -> Result<{model}::Model, ServiceError>;

    async fn delete_{name}_by_identifier(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn count_{name}(&self) -> Result<i64, ServiceError>;
}}

impl<R: {Name}RepositoryExt + Send + Sync> {Name}ServiceExt for {Name}Service<R> {{
    async fn create_{name}(
        &self,
        req: &Create{Name}Request,
    ) -> Result<{model}::Model, ServiceError> {{
        Ok(self.repo.create_{name}(req).await?)
    }}

    async fn find_{name}_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<{model}::Model, ServiceError> {{
        Ok(self.repo.find_{name}_by_identifier(identifier).await?)
    }}

    async fn find_all_{name}(&self) -> Result<Vec<{model}::Model>, ServiceError> {{
        Ok(self.repo.find_all_{name}().await?)
    }}

    async fn update_{name}_by_identifier(
        &self,
        identifier: &str,
        req: &Update{Name}Command,
    ) -> Result<{model}::Model, ServiceError> {{
        Ok(self.repo.update_{name}_by_identifier(identifier, req).await?)
    }}

    async fn delete_{name}_by_identifier(&self, identifier: &str) -> Result<(), ServiceError> {{
        Ok(self.repo.delete_{name}_by_identifier(identifier).await?)
    }}

    async fn count_{name}(&self) -> Result<i64, ServiceError> {{
        Ok(self.repo.count_{name}().await?)
    }}
}}
"#,
        name = cfg.name,
        model = cfg.name,
        Name = cfg.name_pascal,
    )
}

pub fn handler(cfg: &ModuleConfig) -> String {
    format!(
        r#"use std::sync::Arc;

use axum::{{
    Json,
    extract::{{Path, State}},
}};
use sanitizer::Sanitizer;
use serde_json::json;

use crate::errors::service_error::ServiceError;
use crate::extractors::{name}::{{Create{Name}Request, Update{Name}Command}};
use crate::middlewares::validator::ValidatedRequest;
use crate::models::{model};
use crate::services::{name}_services::{Name}ServiceExt;
use crate::state::AppState;

pub async fn create_{name}(
    State(state): State<Arc<AppState>>,
    ValidatedRequest(mut request): ValidatedRequest<Create{Name}Request>,
) -> Result<Json<{model}::Model>, ServiceError> {{
    request.sanitize();

    let result = state
        .services
        .{service_field}
        .create_{name}(&request)
        .await?;

    Ok(Json(result))
}}

pub async fn find_{name}_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<{model}::Model>, ServiceError> {{
    let result = state
        .services
        .{service_field}
        .find_{name}_by_identifier(&identifier)
        .await?;

    Ok(Json(result))
}}

pub async fn find_all_{name}(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<{model}::Model>>, ServiceError> {{
    let results = state
        .services
        .{service_field}
        .find_all_{name}()
        .await?;

    Ok(Json(results))
}}

pub async fn count_{name}(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ServiceError> {{
    let count = state
        .services
        .{service_field}
        .count_{name}()
        .await?;

    Ok(Json(json!({{ "count": count }})))
}}

pub async fn update_{name}_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
    ValidatedRequest(mut request): ValidatedRequest<Update{Name}Command>,
) -> Result<Json<{model}::Model>, ServiceError> {{
    request.sanitize();

    let result = state
        .services
        .{service_field}
        .update_{name}_by_identifier(&identifier, &request)
        .await?;

    Ok(Json(result))
}}

pub async fn delete_{name}_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {{
    state
        .services
        .{service_field}
        .delete_{name}_by_identifier(&identifier)
        .await?;

    Ok(Json(
        json!({{ "message": "{Name} deleted successfully" }}),
    ))
}}
"#,
        name = cfg.name,
        model = cfg.name,
        Name = cfg.name_pascal,
        service_field = format_args!("{}_service", cfg.name),
    )
}

pub fn extractor(cfg: &ModuleConfig) -> String {
    format!(
        r#"use sanitizer::prelude::*;
use serde::{{Deserialize, Serialize}};
use ts_rs::TS;
use validator::Validate;

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Create{Name}Request {{}}

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Update{Name}Command {{}}
"#,
        Name = cfg.name_pascal,
    )
}

pub fn route(cfg: &ModuleConfig) -> String {
    format!(
        r#"use std::sync::Arc;

use axum::middleware;
use axum::routing::{{delete, post, put}};
use axum::{{Router, routing::get}};

use crate::handlers::{handler_file}::{{
    count_{name}, create_{name}, delete_{name}_by_identifier, find_all_{name},
    find_{name}_by_identifier, update_{name}_by_identifier,
}};
use crate::middlewares::auth::authenticate;
use crate::state::AppState;

pub(super) fn {route_fn}(state: Arc<AppState>) -> Router {{
    let routes = Router::new()
        .route("/", post(create_{name}))
        .route("/", get(find_all_{name}))
        .route("/{{identifier}}", get(find_{name}_by_identifier))
        .route("/count", get(count_{name}))
        .route("/{{identifier}}", put(update_{name}_by_identifier))
        .route("/{{identifier}}", delete(delete_{name}_by_identifier));

    Router::new()
        .nest("/{name}", routes)
        .layer(middleware::from_fn(authenticate))
        .with_state(state)
}}
"#,
        handler_file = cfg.name,
        route_fn = format_args!("{}_routes", cfg.name),
        name = cfg.name,
    )
}
