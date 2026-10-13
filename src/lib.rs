#![allow(async_fn_in_trait)]

pub mod config;
pub mod dto;
pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;
pub mod utils;

use crate::models::sea_orm_active_enums::FileType;

impl From<String> for FileType {
    fn from(value: String) -> Self {
        match value.as_str() {
            _ => FileType::Others,
        }
    }
}
