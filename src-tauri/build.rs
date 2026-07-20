//! Build script that compiles YAML tweak definitions into Rust code at compile time.
//!
//! This eliminates runtime YAML parsing and file I/O, making tweak loading instant.
//! When YAML files change, Cargo automatically rebuilds thanks to `rerun-if-changed`.

use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Write;
use std::path::Path;

// ============================================================================
// Shared YAML-schema types
// ============================================================================
//
// The YAML-schema type definitions are shared verbatim with the runtime crate: the single source
// is src/models/tweak_schema.rs, included here via #[path] so build.rs and the runtime can no
// longer drift silently — a renamed field is now a compile error on both sides. build.rs keeps its
// OWN build-only pieces below: TweakDefinitionRaw (the authored form), TweakFile, the validation
// impls, and the raw -> final conversion in generate_tweak_data.
#[path = "src/models/tweak_schema.rs"]
mod tweak_schema;
use tweak_schema::*;

/// Raw tweak definition as loaded from YAML
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TweakDefinitionRaw {
    id: String,
    name: String,
    description: String,
    #[serde(default)]
    info: Option<String>,
    risk_level: RiskLevel,
    #[serde(default)]
    requires_admin: bool,
    #[serde(default)]
    requires_system: bool,
    /// If true, run as TrustedInstaller (for protected services like WaaSMedicSvc)
    #[serde(default)]
    requires_ti: bool,
    #[serde(default)]
    requires_reboot: bool,
    #[serde(default)]
    force_dropdown: bool,
    options: Vec<TweakOption>,
}

/// YAML file structure with category and tweaks
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TweakFile {
    category: CategoryDefinition,
    tweaks: Vec<TweakDefinitionRaw>,
}

// ============================================================================
// Validation Engine
// ============================================================================
// Type-driven validation that catches errors at build time.
// Adding new fields to structs automatically gets basic validation via serde.
// Custom rules here validate semantic correctness beyond type checking.

use std::collections::HashSet;

/// Validation context tracking state across multiple files
struct ValidationContext {
    /// All tweak IDs seen so far (for duplicate detection)
    seen_tweak_ids: HashSet<String>,
    /// All category IDs seen (with file name for error reporting)
    seen_category_ids: HashMap<String, String>,
    /// Collected validation errors (fatal - fail build)
    errors: Vec<String>,
    /// Collected warnings (non-fatal - just report)
    warnings: Vec<String>,
}

impl ValidationContext {
    fn new() -> Self {
        Self {
            seen_tweak_ids: HashSet::new(),
            seen_category_ids: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error with file context (for category-level errors)
    fn error(&mut self, file: &str, msg: String) {
        self.errors.push(format!("[{}] {}", file, msg));
    }

    /// Add an error with file and tweak context
    fn tweak_error(&mut self, file: &str, tweak_id: &str, msg: String) {
        self.errors
            .push(format!("[{}] Tweak '{}': {}", file, tweak_id, msg));
    }

    /// Add a warning with file and tweak context (non-fatal)
    fn tweak_warning(&mut self, file: &str, tweak_id: &str, msg: String) {
        self.warnings
            .push(format!("[{}] Tweak '{}': {}", file, tweak_id, msg));
    }

    /// Check for duplicate category ID, returns true if duplicate found
    fn check_category_duplicate(&mut self, file: &str, category_id: &str) -> bool {
        if let Some(existing_file) = self.seen_category_ids.get(category_id) {
            self.error(
                file,
                format!(
                    "Duplicate category ID '{}' (already defined in {})",
                    category_id, existing_file
                ),
            );
            true
        } else {
            self.seen_category_ids
                .insert(category_id.to_string(), file.to_string());
            false
        }
    }

    /// Validate category definition fields
    fn validate_category(&mut self, file: &str, category: &CategoryDefinition) {
        // Validate category ID format (snake_case)
        if !is_valid_tweak_id(&category.id) {
            self.error(
                file,
                format!(
                    "category ID '{}' must be snake_case (lowercase letters, digits, underscores only)",
                    category.id
                ),
            );
        }

        // Validate required string fields are not empty
        if category.name.trim().is_empty() {
            self.error(file, "category name cannot be empty".to_string());
        }
        if category.description.trim().is_empty() {
            self.error(file, "category description cannot be empty".to_string());
        }
        if category.icon.trim().is_empty() {
            self.error(file, "category icon cannot be empty".to_string());
        }
    }

    /// Check if there are any errors
    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Print warnings to cargo output
    fn print_warnings(&self) {
        for warning in &self.warnings {
            println!("cargo:warning=⚠ {}", warning);
        }
    }

    /// Get formatted error report
    fn error_report(&self) -> String {
        let mut report =
            String::from("\n╔══════════════════════════════════════════════════════════════╗\n");
        report.push_str("║           YAML TWEAK VALIDATION FAILED                       ║\n");
        report.push_str("╠══════════════════════════════════════════════════════════════╣\n");
        report.push_str(&format!(
            "║ {:.<62}║\n",
            format!("{} error(s) found:", self.errors.len())
        ));
        report.push_str("╠══════════════════════════════════════════════════════════════╣\n");
        for (i, error) in self.errors.iter().enumerate() {
            // Errors may span multiple lines; each line needs proper framing
            let numbered = format!("{}. {}", i + 1, error);
            for line in numbered.lines() {
                report.push_str(&format!("║ {}\n", line));
            }
        }
        report.push_str("╚══════════════════════════════════════════════════════════════╝\n");
        report
    }
}

/// Validate tweak ID format (snake_case convention)
fn is_valid_tweak_id(id: &str) -> bool {
    if id.is_empty() {
        return false;
    }
    // Must start with lowercase letter or underscore
    let mut chars = id.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_lowercase() && first != '_' {
        return false;
    }
    // Rest must be lowercase, digits, or underscore
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Valid Windows versions for filtering
const VALID_WINDOWS_VERSIONS: &[u32] = &[10, 11];

impl RegistryChange {
    /// Validate registry change semantic correctness
    fn validate(
        &self,
        ctx: &mut ValidationContext,
        file: &str,
        tweak_id: &str,
        option_label: &str,
    ) {
        let location = format!(
            "option '{}' registry change '{}'",
            option_label, self.value_name
        );

        // Validate key is not empty
        if self.key.trim().is_empty() {
            ctx.tweak_error(
                file,
                tweak_id,
                format!("{}: registry key cannot be empty", location),
            );
        }

        // Action-specific validation
        match self.action {
            RegistryAction::Set => {
                // Set action requires value_type and value
                if self.value_type.is_none() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!("{}: 'set' action requires value_type", location),
                    );
                }
                if self.value.is_none() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!("{}: 'set' action requires value", location),
                    );
                }
                // Validate value_name (empty string targets default value)
                if self.value_name.is_empty() {
                    ctx.tweak_warning(
                        file,
                        tweak_id,
                        format!(
                            "{}: value_name is empty (targeting default value)",
                            location
                        ),
                    );
                } else if self.value_name.trim().is_empty() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!(
                            "{}: value_name is whitespace-only (use empty string for default value)",
                            location
                        ),
                    );
                }
                // Validate value matches value_type (if both present)
                if self.value_type.is_some() && self.value.is_some() {
                    self.validate_value_type(ctx, file, tweak_id, &location);
                }
            }
            RegistryAction::DeleteValue => {
                // DeleteValue requires value_name
                if self.value_name.is_empty() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!("{}: 'delete_value' action requires value_name", location),
                    );
                } else if self.value_name.trim().is_empty() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!("{}: value_name is whitespace-only", location),
                    );
                }
                // Warn if value_type/value are provided (they're ignored)
                if self.value_type.is_some() || self.value.is_some() {
                    ctx.tweak_warning(
                        file,
                        tweak_id,
                        format!(
                            "{}: value_type and value are ignored for 'delete_value' action",
                            location
                        ),
                    );
                }
            }
            RegistryAction::DeleteKey => {
                // DeleteKey only needs the key path
                // Warn if value_name/value_type/value are provided
                if !self.value_name.is_empty() {
                    ctx.tweak_warning(
                        file,
                        tweak_id,
                        format!(
                            "{}: value_name is ignored for 'delete_key' action",
                            location
                        ),
                    );
                }
                if self.value_type.is_some() || self.value.is_some() {
                    ctx.tweak_warning(
                        file,
                        tweak_id,
                        format!(
                            "{}: value_type and value are ignored for 'delete_key' action",
                            location
                        ),
                    );
                }
            }
            RegistryAction::CreateKey => {
                // CreateKey only needs the key path
                // Warn if value_name/value_type/value are provided
                if !self.value_name.is_empty() {
                    ctx.tweak_warning(
                        file,
                        tweak_id,
                        format!(
                            "{}: value_name is ignored for 'create_key' action",
                            location
                        ),
                    );
                }
                if self.value_type.is_some() || self.value.is_some() {
                    ctx.tweak_warning(
                        file,
                        tweak_id,
                        format!(
                            "{}: value_type and value are ignored for 'create_key' action",
                            location
                        ),
                    );
                }
            }
        }

        // Validate Windows versions (applies to all actions)
        if let Some(versions) = &self.windows_versions {
            for v in versions {
                if !VALID_WINDOWS_VERSIONS.contains(v) {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!(
                            "{}: invalid windows_version {}, must be one of {:?}",
                            location, v, VALID_WINDOWS_VERSIONS
                        ),
                    );
                }
            }
        }
    }

    /// Check if this registry change targets HKLM (requires admin)
    fn requires_admin(&self) -> bool {
        matches!(self.hive, RegistryHive::Hklm)
    }

    /// Validate that the value matches the declared value_type
    fn validate_value_type(
        &self,
        ctx: &mut ValidationContext,
        file: &str,
        tweak_id: &str,
        location: &str,
    ) {
        let value_type = match &self.value_type {
            Some(vt) => vt,
            None => return,
        };
        let value = match &self.value {
            Some(v) => v,
            None => return,
        };

        match value_type {
            RegistryValueType::Dword => {
                if !value.is_u64() && !value.is_i64() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!(
                            "{}: REG_DWORD requires integer value, got {}",
                            location,
                            value_type_name(value)
                        ),
                    );
                } else if let Some(n) = value.as_u64() {
                    if n > u32::MAX as u64 {
                        ctx.tweak_error(
                            file,
                            tweak_id,
                            format!(
                                "{}: REG_DWORD value {} exceeds u32::MAX ({})",
                                location,
                                n,
                                u32::MAX
                            ),
                        );
                    }
                } else if let Some(n) = value.as_i64() {
                    if n < 0 || n > u32::MAX as i64 {
                        ctx.tweak_error(
                            file,
                            tweak_id,
                            format!(
                                "{}: REG_DWORD value {} out of range (0..{})",
                                location,
                                n,
                                u32::MAX
                            ),
                        );
                    }
                }
            }
            RegistryValueType::Qword => {
                if !value.is_u64() && !value.is_i64() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!(
                            "{}: REG_QWORD requires integer value, got {}",
                            location,
                            value_type_name(value)
                        ),
                    );
                } else if let Some(n) = value.as_i64() {
                    // REG_QWORD is unsigned (0 to u64::MAX), negative values are invalid
                    if n < 0 {
                        ctx.tweak_error(
                            file,
                            tweak_id,
                            format!(
                                "{}: REG_QWORD value {} is negative; must be in range 0..{}",
                                location,
                                n,
                                u64::MAX
                            ),
                        );
                    }
                }
            }
            RegistryValueType::String | RegistryValueType::ExpandString => {
                if !value.is_string() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!(
                            "{}: {} requires string value, got {}",
                            location,
                            if matches!(value_type, RegistryValueType::String) {
                                "REG_SZ"
                            } else {
                                "REG_EXPAND_SZ"
                            },
                            value_type_name(value)
                        ),
                    );
                }
            }
            RegistryValueType::MultiString => {
                if !value.is_array() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!(
                            "{}: REG_MULTI_SZ requires array of strings, got {}",
                            location,
                            value_type_name(value)
                        ),
                    );
                } else if let Some(arr) = value.as_array() {
                    for (i, item) in arr.iter().enumerate() {
                        if !item.is_string() {
                            ctx.tweak_error(
                                file,
                                tweak_id,
                                format!(
                                    "{}: REG_MULTI_SZ array item [{}] must be string, got {}",
                                    location,
                                    i,
                                    value_type_name(item)
                                ),
                            );
                        }
                    }
                }
            }
            RegistryValueType::Binary => {
                // Binary can be array of integers (bytes) or hex string
                if value.is_array() {
                    if let Some(arr) = value.as_array() {
                        for (i, item) in arr.iter().enumerate() {
                            if let Some(n) = item.as_u64() {
                                if n > 255 {
                                    ctx.tweak_error(
                                        file,
                                        tweak_id,
                                        format!(
                                            "{}: REG_BINARY array item [{}] value {} exceeds byte range (0-255)",
                                            location, i, n
                                        ),
                                    );
                                }
                            } else {
                                ctx.tweak_error(
                                    file,
                                    tweak_id,
                                    format!(
                                        "{}: REG_BINARY array item [{}] must be integer (0-255), got {}",
                                        location,
                                        i,
                                        value_type_name(item)
                                    ),
                                );
                            }
                        }
                    }
                } else if !value.is_string() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!(
                            "{}: REG_BINARY requires array of bytes or hex string, got {}",
                            location,
                            value_type_name(value)
                        ),
                    );
                }
            }
        }
    }
}

impl ServiceChange {
    /// Validate service change semantic correctness
    fn validate(
        &self,
        ctx: &mut ValidationContext,
        file: &str,
        tweak_id: &str,
        option_label: &str,
    ) {
        let location = format!("option '{}' service change", option_label);

        // Validate service name is not empty
        if self.name.trim().is_empty() {
            ctx.tweak_error(
                file,
                tweak_id,
                format!("{}: service name cannot be empty", location),
            );
        }
    }
}

impl SchedulerChange {
    /// Validate scheduler change semantic correctness
    fn validate(
        &self,
        ctx: &mut ValidationContext,
        file: &str,
        tweak_id: &str,
        option_label: &str,
    ) {
        let location = format!("option '{}' scheduler change", option_label);

        // Validate task_path is not empty
        if self.task_path.trim().is_empty() {
            ctx.tweak_error(
                file,
                tweak_id,
                format!("{}: task_path cannot be empty", location),
            );
        }

        // Validate mutual exclusivity: task_name XOR task_name_pattern
        match (&self.task_name, &self.task_name_pattern) {
            (None, None) => {
                ctx.tweak_error(
                    file,
                    tweak_id,
                    format!(
                        "{}: must specify either 'task_name' or 'task_name_pattern'",
                        location
                    ),
                );
            }
            (Some(_), Some(_)) => {
                ctx.tweak_error(
                    file,
                    tweak_id,
                    format!(
                        "{}: cannot specify both 'task_name' and 'task_name_pattern' (mutually exclusive)",
                        location
                    ),
                );
            }
            (Some(name), None) => {
                // Validate task_name is not empty
                if name.trim().is_empty() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!("{}: task_name cannot be empty", location),
                    );
                }
            }
            (None, Some(pattern)) => {
                // Validate task_name_pattern is not empty
                if pattern.trim().is_empty() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!("{}: task_name_pattern cannot be empty", location),
                    );
                }
                // Validate task_name_pattern is valid regex
                else if let Err(regex_err) = regex::Regex::new(pattern) {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!(
                            "{}: invalid regex pattern '{}' in task_name_pattern: {}",
                            location, pattern, regex_err
                        ),
                    );
                }
            }
        }
    }
}

impl HostsChange {
    /// Validate hosts change semantic correctness
    fn validate(
        &self,
        ctx: &mut ValidationContext,
        file: &str,
        tweak_id: &str,
        option_label: &str,
    ) {
        let location = format!("option '{}' hosts change", option_label);

        if self.ip.trim().is_empty() {
            ctx.tweak_error(file, tweak_id, format!("{}: ip cannot be empty", location));
        }
        if self.domain.trim().is_empty() {
            ctx.tweak_error(
                file,
                tweak_id,
                format!("{}: domain cannot be empty", location),
            );
        }
    }
}

impl FirewallChange {
    /// Validate firewall change semantic correctness
    fn validate(
        &self,
        ctx: &mut ValidationContext,
        file: &str,
        tweak_id: &str,
        option_label: &str,
    ) {
        let location = format!("option '{}' firewall change '{}'", option_label, self.name);

        if self.name.trim().is_empty() {
            ctx.tweak_error(
                file,
                tweak_id,
                format!("{}: firewall rule name cannot be empty", location),
            );
        }

        // Create operation requires direction and action
        if matches!(self.operation, FirewallOperation::Create) {
            if self.direction.is_none() {
                ctx.tweak_error(
                    file,
                    tweak_id,
                    format!("{}: 'create' operation requires 'direction'", location),
                );
            }
            if self.action.is_none() {
                ctx.tweak_error(
                    file,
                    tweak_id,
                    format!("{}: 'create' operation requires 'action'", location),
                );
            }
        }
    }
}

impl TweakOption {
    /// Validate option semantic correctness
    fn validate(&self, ctx: &mut ValidationContext, file: &str, tweak_id: &str) {
        // Validate option label is not empty or whitespace
        if self.label.trim().is_empty() {
            ctx.tweak_error(
                file,
                tweak_id,
                "option label cannot be empty or whitespace-only".to_string(),
            );
        }

        // Validate all registry changes
        for change in &self.registry_changes {
            change.validate(ctx, file, tweak_id, &self.label);
        }

        // Validate all service changes
        for change in &self.service_changes {
            change.validate(ctx, file, tweak_id, &self.label);
        }

        // Validate all scheduler changes
        for change in &self.scheduler_changes {
            change.validate(ctx, file, tweak_id, &self.label);
        }

        // Validate all hosts changes
        for change in &self.hosts_changes {
            change.validate(ctx, file, tweak_id, &self.label);
        }

        // Validate all firewall changes
        for change in &self.firewall_changes {
            change.validate(ctx, file, tweak_id, &self.label);
        }

        // Check for empty option (no changes at all)
        let has_any_changes = !self.registry_changes.is_empty()
            || !self.service_changes.is_empty()
            || !self.scheduler_changes.is_empty()
            || !self.hosts_changes.is_empty()
            || !self.firewall_changes.is_empty()
            || !self.pre_commands.is_empty()
            || !self.post_commands.is_empty()
            || !self.pre_powershell.is_empty()
            || !self.post_powershell.is_empty();

        if !has_any_changes {
            ctx.tweak_error(
                file,
                tweak_id,
                format!(
                    "option '{}' has no changes (registry, service, scheduler, hosts, firewall, or commands)",
                    self.label
                ),
            );
        }
    }

    /// Check if this option requires admin privileges (any HKLM registry change)
    fn requires_admin(&self) -> bool {
        self.registry_changes.iter().any(|r| r.requires_admin())
    }
}

impl TweakDefinitionRaw {
    /// Validate tweak definition semantic correctness
    fn validate(&self, ctx: &mut ValidationContext, file: &str) {
        // Validate tweak ID format (snake_case)
        if !is_valid_tweak_id(&self.id) {
            ctx.tweak_error(
                file,
                &self.id,
                "tweak ID must be snake_case (lowercase letters, digits, underscores only)"
                    .to_string(),
            );
        }

        // Check for duplicate ID
        if ctx.seen_tweak_ids.contains(&self.id) {
            ctx.tweak_error(
                file,
                &self.id,
                "duplicate tweak ID (already defined in another file)".to_string(),
            );
        } else {
            ctx.seen_tweak_ids.insert(self.id.clone());
        }

        // Validate option count (minimum 2 required)
        if self.options.len() < 2 {
            ctx.tweak_error(
                file,
                &self.id,
                format!("must have at least 2 options, found {}", self.options.len()),
            );
            return; // Can't validate further without proper options
        }

        // Warn if force_dropdown used with 3+ options (unnecessary)
        if self.force_dropdown && self.options.len() > 2 {
            ctx.tweak_warning(
                file,
                &self.id,
                "force_dropdown is unnecessary for 3+ options (already defaults to dropdown)"
                    .to_string(),
            );
        }

        // Check for duplicate option labels within this tweak
        let mut seen_labels: HashSet<String> = HashSet::new();
        for option in &self.options {
            let label_lower = option.label.to_lowercase();
            if seen_labels.contains(&label_lower) {
                ctx.tweak_error(
                    file,
                    &self.id,
                    format!(
                        "duplicate option label '{}' (case-insensitive)",
                        option.label
                    ),
                );
            } else {
                seen_labels.insert(label_lower);
            }
        }

        // Validate each option
        for option in &self.options {
            option.validate(ctx, file, &self.id);
        }

        // Check if any option requires admin but tweak doesn't declare it
        let any_requires_admin = self.options.iter().any(|o| o.requires_admin());
        if any_requires_admin && !self.requires_admin && !self.requires_system && !self.requires_ti
        {
            ctx.tweak_warning(
                file,
                &self.id,
                "contains HKLM registry changes but requires_admin is false (should be true)"
                    .to_string(),
            );
        }
    }
}

/// Get human-readable name for JSON value type
fn value_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

// ============================================================================
// Build script main
// ============================================================================

fn main() {
    // Standard Tauri build
    tauri_build::build();

    // Generate tweak data from YAML files
    if let Err(e) = generate_tweak_data() {
        panic!("Failed to generate tweak data: {}", e);
    }
}

fn generate_tweak_data() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let tweaks_dir = Path::new(&manifest_dir).join("tweaks");
    let out_dir = std::env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    // Tell Cargo to rerun if any YAML file changes
    println!("cargo:rerun-if-changed=tweaks/");
    for entry in fs::read_dir(&tweaks_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    // Initialize validation context
    let mut validation_ctx = ValidationContext::new();

    // Collect all categories and tweaks
    let mut categories: Vec<CategoryDefinition> = Vec::new();
    // BTreeMap, not HashMap: this map is serialized straight to tweaks.json, and
    // HashMap's randomly-seeded hasher emits the keys in a different order on every
    // build. That made the generated artifact differ byte-for-byte between builds of
    // identical input, which defeats build caching and makes it impossible to verify
    // that a change to the parser or the YAML left the output untouched.
    // The runtime still deserializes into a HashMap; JSON object order is irrelevant there.
    let mut tweaks: BTreeMap<String, TweakDefinition> = BTreeMap::new();
    let mut parse_errors: Vec<String> = Vec::new();

    // First pass: parse all files and collect categories
    let mut parsed_files: Vec<(String, TweakFile)> = Vec::new();

    for entry in fs::read_dir(&tweaks_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }

        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(&path)?;

        let tweak_file: TweakFile = match serde_yaml_bw::from_str(&content) {
            Ok(tf) => tf,
            Err(e) => {
                parse_errors.push(format!("[{}] Parse error: {}", file_name, e));
                continue;
            }
        };

        // Check for duplicate category ID and validate category fields
        validation_ctx.check_category_duplicate(&file_name, &tweak_file.category.id);
        validation_ctx.validate_category(&file_name, &tweak_file.category);
        parsed_files.push((file_name, tweak_file));
    }

    // Report parse errors first (these are fatal, prevent further validation)
    if !parse_errors.is_empty() {
        let mut report =
            String::from("\n╔══════════════════════════════════════════════════════════════╗\n");
        report.push_str("║           YAML PARSE ERRORS                                  ║\n");
        report.push_str("╠══════════════════════════════════════════════════════════════╣\n");
        for error in &parse_errors {
            // Errors may span multiple lines; each line needs proper framing
            for line in error.lines() {
                report.push_str(&format!("║ {}\n", line));
            }
        }
        report.push_str("╚══════════════════════════════════════════════════════════════╝\n");
        return Err(report.into());
    }

    // Second pass: validate and build tweaks
    for (file_name, tweak_file) in parsed_files {
        let category_id = tweak_file.category.id.clone();
        categories.push(tweak_file.category);

        for raw in tweak_file.tweaks {
            // Run semantic validation
            raw.validate(&mut validation_ctx, &file_name);

            // Build tweak definition with permission inference
            let requires_ti = raw.requires_ti;
            let requires_system = raw.requires_system || requires_ti;
            let requires_admin = raw.requires_admin || requires_system;

            let tweak = TweakDefinition {
                id: raw.id.clone(),
                name: raw.name,
                description: raw.description,
                info: raw.info,
                risk_level: raw.risk_level,
                requires_admin,
                requires_system,
                requires_ti,
                requires_reboot: raw.requires_reboot,
                force_dropdown: raw.force_dropdown,
                options: raw.options,
                category_id: category_id.clone(),
            };
            tweaks.insert(raw.id, tweak);
        }
    }

    // Print any warnings (non-fatal)
    validation_ctx.print_warnings();

    // Report validation errors with detailed report
    if validation_ctx.has_errors() {
        return Err(validation_ctx.error_report().into());
    }

    // Sort categories by order
    categories.sort_by_key(|c| c.order);

    // Write JSON files separately (avoids escaping issues)
    let categories_json_path = out_path.join("categories.json");
    let tweaks_json_path = out_path.join("tweaks.json");

    fs::write(&categories_json_path, serde_json::to_string(&categories)?)?;
    fs::write(&tweaks_json_path, serde_json::to_string(&tweaks)?)?;

    // Generate Rust code that includes the JSON files
    let generated_code = format!(
        r#"// AUTO-GENERATED FILE - DO NOT EDIT
// Generated from YAML files in tweaks/ directory at build time.
// To modify tweaks, edit the YAML files and rebuild.

use std::collections::HashMap;
use std::sync::LazyLock;
use crate::models::{{CategoryDefinition, TweakDefinition}};

/// Raw JSON string of categories (embedded at compile time)
pub const CATEGORIES_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/categories.json"));

/// Raw JSON string of tweaks (embedded at compile time)
pub const TWEAKS_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/tweaks.json"));

/// Pre-compiled categories loaded from YAML at build time.
/// Sorted by `order` field.
pub static CATEGORIES: LazyLock<Vec<CategoryDefinition>> = LazyLock::new(|| {{
    serde_json::from_str(CATEGORIES_JSON).expect("Failed to parse embedded categories JSON")
}});

/// Pre-compiled tweaks loaded from YAML at build time.
/// HashMap for O(1) lookup by tweak ID.
pub static TWEAKS: LazyLock<HashMap<String, TweakDefinition>> = LazyLock::new(|| {{
    serde_json::from_str(TWEAKS_JSON).expect("Failed to parse embedded tweaks JSON")
}});

/// Number of categories compiled into the binary
#[allow(dead_code)]
pub const CATEGORY_COUNT: usize = {category_count};

"#,
        category_count = categories.len(),
    );

    // Write the generated Rust file
    let rust_path = out_path.join("generated_tweaks.rs");
    let mut file = fs::File::create(&rust_path)?;
    file.write_all(generated_code.as_bytes())?;

    println!(
        "cargo:warning=✓ Validated and generated {} categories and {} tweaks from YAML files",
        categories.len(),
        tweaks.len()
    );

    Ok(())
}
