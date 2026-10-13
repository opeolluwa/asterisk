use std::fs;
use std::path::{Path, PathBuf};

use super::ModuleConfig;
use crate::errors::CliError;
use crate::logging::LogMessage;
use crate::templates::backend;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("generator manifest directory must have a parent")
        .to_path_buf()
}

struct GenerationPaths {
    root: PathBuf,
    repository: PathBuf,
    service: PathBuf,
    extractor: PathBuf,
    handler: PathBuf,
    route: PathBuf,
    repository_mod: PathBuf,
    service_mod: PathBuf,
    extractor_mod: PathBuf,
    handler_mod: PathBuf,
    route_mod: PathBuf,
    router: PathBuf,
    state_repositories: PathBuf,
    state_services: PathBuf,
}

impl GenerationPaths {
    fn new(root: &Path, cfg: &ModuleConfig) -> Self {
        let repositories = root.join("src/repositories");
        let services = root.join("src/services");
        let extractors = root.join("src/extractors");
        let handlers = root.join("src/handlers");
        let routes = root.join("src/routes");
        let state = root.join("src/state");

        Self {
            root: root.to_path_buf(),
            repository: repositories.join(format!("{}_repository.rs", cfg.name)),
            service: services.join(format!("{}_services.rs", cfg.name)),
            extractor: extractors.join(format!("{}.rs", cfg.name)),
            handler: handlers.join(format!("{}.rs", cfg.name)),
            route: routes.join(format!("{}.rs", cfg.name)),
            repository_mod: repositories.join("mod.rs"),
            service_mod: services.join("mod.rs"),
            extractor_mod: extractors.join("mod.rs"),
            handler_mod: handlers.join("mod.rs"),
            route_mod: routes.join("mod.rs"),
            router: routes.join("router.rs"),
            state_repositories: state.join("repositories.rs"),
            state_services: state.join("services.rs"),
        }
    }

    fn generated_files(&self) -> [&Path; 5] {
        [
            &self.repository,
            &self.service,
            &self.extractor,
            &self.handler,
            &self.route,
        ]
    }
}

fn conflict(message: impl Into<String>) -> CliError {
    CliError::OperationFailed(message.into())
}

fn read_file(path: &Path) -> Result<String, CliError> {
    fs::read_to_string(path)
        .map_err(|e| CliError::IoError(format!("Failed to read {}: {e}", path.display())))
}

fn write_file(path: &Path, content: &str) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            CliError::IoError(format!("Failed to create dir {}: {e}", parent.display()))
        })?;
    }
    fs::write(path, content)
        .map_err(|e| CliError::IoError(format!("Failed to write {}: {e}", path.display())))?;
    Ok(())
}

fn write_new_file(path: &Path, content: &str) -> Result<(), CliError> {
    if path.exists() {
        return Err(conflict(format!(
            "Refusing to overwrite existing file: {}",
            path.display()
        )));
    }
    write_file(path, content)
}

fn ensure_targets_absent(paths: &GenerationPaths) -> Result<(), CliError> {
    for path in paths.generated_files() {
        if path.exists() {
            return Err(conflict(format!(
                "Refusing to overwrite existing file: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn join_lines(lines: &[String]) -> String {
    let mut content = lines.join("\n");
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content
}

fn append_to_mod_content(content: &str, entry: &str) -> Result<String, CliError> {
    let declaration = format!("pub mod {entry};");

    if content.lines().any(|line| line.trim() == declaration) {
        return Err(conflict(format!(
            "Module declaration already exists: {declaration}"
        )));
    }

    let mut new_content = content.trim_end().to_string();
    new_content.push('\n');
    new_content.push_str(&declaration);
    new_content.push('\n');
    Ok(new_content)
}

fn router_entries(content: &str, cfg: &ModuleConfig) -> (bool, bool) {
    let route_fn = format!("{}_routes", cfg.name);
    let import = format!("{}::{},", cfg.name, route_fn);
    let merge = format!(".merge({}(Arc::clone(&state))),", route_fn);

    (
        content.lines().any(|line| line.trim() == import),
        content.lines().any(|line| line.trim() == merge),
    )
}

fn update_router_content(content: &str, cfg: &ModuleConfig) -> Result<String, CliError> {
    let route_fn = format!("{}_routes", cfg.name);
    let (has_import, has_merge) = router_entries(content, cfg);

    if has_import || has_merge {
        return Err(conflict(format!(
            "Router already contains configuration for module '{}'",
            cfg.name
        )));
    }

    let import_entry = format!("    {}::{},", cfg.name, route_fn);
    let merge_entry = format!("                .merge({}(Arc::clone(&state))),", route_fn);
    let mut lines: Vec<String> = content.lines().map(ToOwned::to_owned).collect();

    let import_start = lines
        .iter()
        .position(|line| line.trim() == "use crate::routes::{")
        .ok_or_else(|| conflict("Router import group was not found"))?;
    let import_end = lines
        .iter()
        .enumerate()
        .skip(import_start + 1)
        .find_map(|(index, line)| (line.trim() == "};").then_some(index))
        .ok_or_else(|| conflict("Router import group has no closing delimiter"))?;
    lines.insert(import_end, import_entry);

    let last_merge = lines
        .iter()
        .rposition(|line| line.trim_start().starts_with(".merge("))
        .ok_or_else(|| conflict("Router merge chain was not found"))?;
    lines.insert(last_merge + 1, merge_entry);

    Ok(join_lines(&lines))
}

struct StateEntries {
    repository_import: String,
    repository_field: String,
    repository_init: String,
    service_import: String,
    service_field: String,
    service_init: String,
}

fn state_entries(cfg: &ModuleConfig) -> StateEntries {
    let repository_field = format!("{}_repository", cfg.name);
    let service_field = format!("{}_service", cfg.name);
    let repository_struct = format!("{}Repository", cfg.name_pascal);
    let service_struct = format!("{}Service", cfg.name_pascal);

    StateEntries {
        repository_import: format!("{repository_field}::{repository_struct},"),
        repository_field: format!("pub {repository_field}: {repository_struct},"),
        repository_init: format!("{repository_field}: {repository_struct}::init(db),"),
        service_import: format!("{}_services::{service_struct},", cfg.name),
        service_field: format!("pub {service_field}: {service_struct}<{repository_struct}>,"),
        service_init: format!("{service_field}: {service_struct}::new(repos.{repository_field}),"),
    }
}

fn contains_line(content: &str, expected: &str) -> bool {
    content.lines().any(|line| line.trim() == expected)
}

fn state_has_entries(content: &str, entries: &StateEntries) -> bool {
    [
        entries.repository_import.as_str(),
        entries.repository_field.as_str(),
        entries.repository_init.as_str(),
        entries.service_import.as_str(),
        entries.service_field.as_str(),
        entries.service_init.as_str(),
    ]
    .iter()
    .any(|entry| contains_line(content, entry))
}

fn find_region(lines: &[String], marker: &str) -> Option<(usize, usize)> {
    let start = lines.iter().position(|line| line.trim() == marker)?;
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| (line.trim() == "}").then_some(index))?;
    Some((start, end))
}

fn insert_in_group(lines: &mut Vec<String>, group: &str, entry: &str) -> Result<(), CliError> {
    let start = lines
        .iter()
        .position(|line| line.trim() == group)
        .ok_or_else(|| conflict(format!("Import group not found: {group}")))?;
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| (line.trim() == "};").then_some(index))
        .ok_or_else(|| conflict(format!("Import group has no closing delimiter: {group}")))?;

    lines.insert(end, format!("    {entry},"));
    Ok(())
}

fn insert_in_region<F>(
    lines: &mut Vec<String>,
    marker: &str,
    entry: &str,
    matches: F,
) -> Result<(), CliError>
where
    F: Fn(&str) -> bool,
{
    let (start, end) = find_region(lines, marker)
        .ok_or_else(|| conflict(format!("Region not found: {marker}")))?;
    let insertion = lines[start..end]
        .iter()
        .rposition(|line| matches(line))
        .map(|index| start + index + 1)
        .unwrap_or(end);

    lines.insert(insertion, entry.to_string());
    Ok(())
}

fn update_repository_state_content(
    content: &str,
    entries: &StateEntries,
) -> Result<String, CliError> {
    if state_has_entries(content, entries) {
        return Err(conflict("Repository state already contains this module"));
    }

    let mut lines: Vec<String> = content.lines().map(ToOwned::to_owned).collect();
    insert_in_group(
        &mut lines,
        "use crate::repositories::{",
        &entries.repository_import,
    )?;
    insert_in_region(
        &mut lines,
        "pub struct Repositories {",
        &entries.repository_field,
        |line| line.trim_start().starts_with("pub "),
    )?;
    insert_in_region(
        &mut lines,
        "impl Repositories {",
        &entries.repository_init,
        |line| line.contains("Repository::init("),
    )?;

    Ok(join_lines(&lines))
}

fn update_services_state_content(
    content: &str,
    entries: &StateEntries,
) -> Result<String, CliError> {
    if state_has_entries(content, entries) {
        return Err(conflict("Service state already contains this module"));
    }

    let mut lines: Vec<String> = content.lines().map(ToOwned::to_owned).collect();
    insert_in_group(
        &mut lines,
        "use crate::repositories::{",
        &entries.repository_import,
    )?;
    insert_in_group(
        &mut lines,
        "use crate::services::{",
        &entries.service_import,
    )?;
    insert_in_region(
        &mut lines,
        "pub struct ServicesState {",
        &entries.service_field,
        |line| line.trim_start().starts_with("pub "),
    )?;
    insert_in_region(
        &mut lines,
        "impl ServicesState {",
        &entries.service_init,
        |line| line.contains("Service::new("),
    )?;

    Ok(join_lines(&lines))
}

fn step(root: &Path, path: &Path, message: &str) {
    LogMessage::step(&format!(
        "{} {}",
        message,
        path.strip_prefix(root).unwrap_or(path).display()
    ));
}

pub fn generate(cfg: &ModuleConfig) -> Result<(), CliError> {
    generate_in(&workspace_root(), cfg)
}

pub(crate) fn generate_in(root: &Path, cfg: &ModuleConfig) -> Result<(), CliError> {
    let paths = GenerationPaths::new(root, cfg);
    ensure_targets_absent(&paths)?;

    let repository_mod_content = read_file(&paths.repository_mod)?;
    let service_mod_content = read_file(&paths.service_mod)?;
    let extractor_mod_content = read_file(&paths.extractor_mod)?;
    let handler_mod_content = read_file(&paths.handler_mod)?;
    let route_mod_content = read_file(&paths.route_mod)?;
    let router_content = read_file(&paths.router)?;
    let state_repositories_content = read_file(&paths.state_repositories)?;
    let state_services_content = read_file(&paths.state_services)?;

    let updated_repository_mod =
        append_to_mod_content(&repository_mod_content, &format!("{}_repository", cfg.name))?;
    let updated_service_mod =
        append_to_mod_content(&service_mod_content, &format!("{}_services", cfg.name))?;
    let updated_extractor_mod = append_to_mod_content(&extractor_mod_content, &cfg.name)?;
    let updated_handler_mod = append_to_mod_content(&handler_mod_content, &cfg.name)?;
    let updated_route_mod = append_to_mod_content(&route_mod_content, &cfg.name)?;
    let updated_router = update_router_content(&router_content, cfg)?;
    let entries = state_entries(cfg);
    let updated_state_repositories =
        update_repository_state_content(&state_repositories_content, &entries)?;
    let updated_state_services = update_services_state_content(&state_services_content, &entries)?;

    write_new_file(&paths.repository, &backend::repository(cfg))?;
    step(&paths.root, &paths.repository, "Created");
    write_new_file(&paths.service, &backend::service(cfg))?;
    step(&paths.root, &paths.service, "Created");
    write_new_file(&paths.extractor, &backend::extractor(cfg))?;
    step(&paths.root, &paths.extractor, "Created");
    write_new_file(&paths.handler, &backend::handler(cfg))?;
    step(&paths.root, &paths.handler, "Created");
    write_new_file(&paths.route, &backend::route(cfg))?;
    step(&paths.root, &paths.route, "Created");

    write_file(&paths.repository_mod, &updated_repository_mod)?;
    write_file(&paths.service_mod, &updated_service_mod)?;
    write_file(&paths.extractor_mod, &updated_extractor_mod)?;
    write_file(&paths.handler_mod, &updated_handler_mod)?;
    write_file(&paths.route_mod, &updated_route_mod)?;
    write_file(&paths.router, &updated_router)?;
    step(&paths.root, &paths.router, "Updated");
    write_file(&paths.state_repositories, &updated_state_repositories)?;
    step(&paths.root, &paths.state_repositories, "Updated");
    write_file(&paths.state_services, &updated_state_services)?;
    step(&paths.root, &paths.state_services, "Updated");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_WORKSPACE: AtomicUsize = AtomicUsize::new(0);

    struct TestWorkspace {
        root: PathBuf,
    }

    impl TestWorkspace {
        fn new() -> Self {
            let id = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "asterisk-generator-test-{}-{id}",
                std::process::id()
            ));
            fs::create_dir_all(&root).unwrap();
            let workspace = Self { root };
            workspace.write_fixture();
            workspace
        }

        fn path(&self) -> &Path {
            &self.root
        }

        fn read(&self, relative_path: &str) -> String {
            fs::read_to_string(self.root.join(relative_path)).unwrap()
        }

        fn write_fixture(&self) {
            for relative_path in [
                "src/repositories",
                "src/services",
                "src/extractors",
                "src/handlers",
                "src/routes",
                "src/state",
            ] {
                fs::create_dir_all(self.root.join(relative_path)).unwrap();
            }

            fs::write(self.root.join("src/repositories/mod.rs"), "pub mod base;\n").unwrap();
            fs::write(
                self.root.join("src/services/mod.rs"),
                "pub mod existing_services;\n",
            )
            .unwrap();
            fs::write(
                self.root.join("src/extractors/mod.rs"),
                "pub mod existing;\n",
            )
            .unwrap();
            fs::write(
                self.root.join("src/handlers/mod.rs"),
                "pub mod existing;\n",
            )
            .unwrap();
            fs::write(
                self.root.join("src/routes/mod.rs"),
                "pub mod existing;\n",
            )
            .unwrap();
            fs::write(
                self.root.join("src/routes/router.rs"),
                "use std::sync::Arc;\n\nuse axum::Router;\n\nuse crate::routes::{\n    existing::existing_routes,\n};\nuse crate::state::AppState;\n\npub fn load_routes(app_state: AppState) -> Router {\n    let state = Arc::new(app_state);\n\n    Router::new()\n        .merge(existing_routes(Arc::clone(&state)))\n}\n",
            )
            .unwrap();
            fs::write(
                self.root.join("src/state/repositories.rs"),
                "use sea_orm::DatabaseConnection;\n\nuse crate::repositories::{\n    base::Repository, existing_repository::ExistingRepository,\n};\n\npub struct Repositories {\n    pub existing: ExistingRepository,\n}\n\nimpl Repositories {\n    pub fn new(db: &DatabaseConnection) -> Self {\n        Self {\n            existing: ExistingRepository::init(db),\n        }\n    }\n}\n",
            )
            .unwrap();
            fs::write(
                self.root.join("src/state/services.rs"),
                "use crate::repositories::{\n    existing_repository::ExistingRepository,\n};\nuse crate::services::{\n    existing_services::ExistingService,\n};\n\npub struct ServicesState {\n    pub existing_service: ExistingService<ExistingRepository>,\n}\n\nimpl ServicesState {\n    pub fn new(repos: crate::state::Repositories) -> Self {\n        Self {\n            existing_service: ExistingService::new(repos.existing),\n        }\n    }\n}\n",
            )
            .unwrap();
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn generates_split_state_wiring() {
        let workspace = TestWorkspace::new();
        let cfg = ModuleConfig::new("inventory", "Inventory management");

        generate_in(workspace.path(), &cfg).unwrap();

        let repository_state = workspace.read("src/state/repositories.rs");
        assert!(repository_state.contains("inventory_repository::InventoryRepository,"));
        assert!(repository_state.contains("pub inventory_repository: InventoryRepository,"));
        assert!(repository_state.contains("inventory_repository: InventoryRepository::init(db),"));

        let service_state = workspace.read("src/state/services.rs");
        assert!(service_state.contains("inventory_repository::InventoryRepository,"));
        assert!(service_state.contains("inventory_services::InventoryService,"));
        assert!(
            service_state.contains("pub inventory_service: InventoryService<InventoryRepository>,")
        );
        assert!(
            service_state
                .contains("inventory_service: InventoryService::new(repos.inventory_repository),")
        );

        let router = workspace.read("src/routes/router.rs");
        assert!(router.contains("inventory::inventory_routes,"));
        assert!(router.contains(".merge(inventory_routes(Arc::clone(&state))),"));
        assert!(
            workspace
                .read("src/repositories/mod.rs")
                .contains("pub mod inventory_repository;")
        );
        assert!(
            workspace
                .read("src/services/mod.rs")
                .contains("pub mod inventory_services;")
        );
        assert!(
            workspace
                .read("src/extractors/mod.rs")
                .contains("pub mod inventory;")
        );
        assert!(
            workspace
                .read("src/handlers/mod.rs")
                .contains("pub mod inventory;")
        );
        assert!(
            workspace
                .read("src/routes/mod.rs")
                .contains("pub mod inventory;")
        );
    }

    #[test]
    fn normalizes_module_names() {
        let cfg = ModuleConfig::new("BlogPost", "");

        assert_eq!(cfg.name, "blog_post");
        assert_eq!(cfg.name_pascal, "BlogPost");
    }

    #[test]
    fn refuses_to_overwrite_existing_files_before_writing() {
        let workspace = TestWorkspace::new();
        let repository_path = workspace
            .path()
            .join("src/repositories/inventory_repository.rs");
        fs::write(&repository_path, "existing content\n").unwrap();
        let cfg = ModuleConfig::new("inventory", "Inventory management");

        let result = generate_in(workspace.path(), &cfg);

        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(repository_path).unwrap(),
            "existing content\n"
        );
        assert!(
            !workspace
                .path()
                .join("src/services/inventory_services.rs")
                .exists()
        );
        assert_eq!(workspace.read("src/repositories/mod.rs"), "pub mod base;\n");
    }

    #[test]
    fn refuses_to_overwrite_on_rerun() {
        let workspace = TestWorkspace::new();
        let cfg = ModuleConfig::new("inventory", "Inventory management");

        generate_in(workspace.path(), &cfg).unwrap();
        let repository = workspace.read("src/repositories/inventory_repository.rs");
        let repository_mod = workspace.read("src/repositories/mod.rs");
        let result = generate_in(workspace.path(), &cfg);

        assert!(result.is_err());
        assert_eq!(
            workspace.read("src/repositories/inventory_repository.rs"),
            repository
        );
        assert_eq!(workspace.read("src/repositories/mod.rs"), repository_mod);
    }

    #[test]
    fn rejects_partial_router_configuration_before_writing() {
        let workspace = TestWorkspace::new();
        fs::write(
            workspace.path().join("src/routes/router.rs"),
            "use crate::routes::{\n    inventory::inventory_routes,\n};\n",
        )
        .unwrap();
        let cfg = ModuleConfig::new("inventory", "Inventory management");

        let result = generate_in(workspace.path(), &cfg);

        assert!(result.is_err());
        assert!(
            !workspace
                .path()
                .join("src/repositories/inventory_repository.rs")
                .exists()
        );
        assert_eq!(workspace.read("src/repositories/mod.rs"), "pub mod base;\n");
    }
}
