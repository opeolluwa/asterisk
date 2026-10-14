alias w := watch
alias k := kill
alias b := build
alias l := logs
alias r := restart
alias cfg := copy-env
alias start := watch
alias run := watch

set dotenv-required := false
set dotenv-load
set dotenv-path := "./.env"
set export

DOCKER_CMD := "docker compose -f docker-compose.yaml"
DEV_DB_URL := "postgres://asterisk:asterisk@localhost:6543/asterisk"
REDIS_CONNECTION_URL := "redis://localhost:7379"

@default:
    @just --list --list-heading $'Available commands\n'

@copy-env:
    cp .env.example .env

#Start dev environment and follow logs
@watch:
    {{ DOCKER_CMD }} up -d
    @just l


#Follow dev app logs
@logs:
    {{ DOCKER_CMD }} logs -f --tail='30' app

#Tear down dev environment and volumes
@kill:
    {{ DOCKER_CMD }} down -v

#Restart dev environment
@restart:
    @just kill
    @just watch

#Build backend binary in release mode
build:
    cargo build --release

#Preview email templates in the browser
@preview-emails:
    node scripts/email-preview.mjs

#Add a new SeaORM migration
migrate-add target:
    sea-orm-cli migrate generate {{ target }}

#Run SeaORM migrations against dev DB
@run-migrations:
    sea-orm-cli migrate up --database-url {{ DEV_DB_URL }}

#Regenerate SeaORM entities from dev DB
@generate-entities:
    RUST_BACKTRACE=full sea-orm-cli generate entity \
        --database-url {{ DEV_DB_URL }} \
        --with-serde both \
        --enum-extra-derives 'ts_rs::TS' \
        --enum-extra-attributes 'serde(rename_all = "snake_case")' \
        --model-extra-attributes 'serde(rename_all="camelCase")' \
        --model-extra-attributes 'macros::ts_rs_export_sea_orm_entity_name' \
        --ignore-tables asterisk_server_migrations \
        -o src/models

#Regenerate frontend TypeScript bindings in typegen/
@export-bindings:
    node scripts/ts-export.js

#Pull DB schema, regenerate entities, and export bindings
db-pull:
    just run-migrations
    just generate-entities
    cargo test
    just export-bindings

#Create a new admin user via CLI
run-cli:
    DATABASE_URL={{ DEV_DB_URL }} REDIS_CONNECTION_URL={{ REDIS_CONNECTION_URL }} cargo run --bin asterisk-cli create-user

#Lint frontend and format backend code
lint:
    cargo sort -w
    cargo group-imports --fix
    cargo fmt
    cargo clippy --fix

sort-env:
    node scripts/sort-env.mjs

env-diff:
    node scripts/env-diff.mjs

#Regenerate docs/openapi.json from the Axum routes and handlers
openapi:
    node scripts/openapi/generate.mjs

#Fail if docs/openapi.json is out of date (for CI)
openapi-check:
    node scripts/openapi/generate.mjs --check

#Browse docs/openapi.json in a Scalar UI, generating it first if missing
openapi-preview:
    node scripts/openapi/preview.mjs

#Generate all SDKs (Go + TypeScript) from docs/openapi.json
sdk:
    node scripts/generate-sdk.js

#Generate Go SDK only (requires oapi-codegen: go install github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@latest)
sdk-go:
    mkdir -p sdk-go
    $(go env GOPATH)/bin/oapi-codegen -package asterisk -generate types,client,spec -o sdk-go/client.go -response-type-suffix Resp docs/openapi.json

#Run backend test suite
test:
    cargo test

#Bump version, tag, and push release
release bump="patch":
    bash scripts/release.sh {{ bump }}

build-prod:
    docker build -f docker/prod/Dockerfile -t asterisk:prod .
    docker images asterisk:prod

prepare:
    go install github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@latest 2>&1