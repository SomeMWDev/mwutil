use crate::config::MWUtilConfig;
use crate::utils::get_core_version;
use std::fmt::Display;

pub fn execute(config: &MWUtilConfig) -> anyhow::Result<()> {
    print_section(
        "Environment",
        &[
            ("Base dir", &config.base_dir.to_string_lossy().to_string()),
            ("Config dir", &config.config_dir.to_string_lossy().to_string()),
            ("Core dir", &config.core_dir.to_string_lossy().to_string()),
            ("Dump dir", &config.dump_dir.to_string_lossy().to_string()),
            ("Profiles", &config.compose_profiles.join(", ")),
        ]
    );
    println!();

    let core_version = get_core_version(config);
    let core_version_display = core_version
        .as_ref()
        .map(|v| v as &dyn Display)
        .unwrap_or(&"Unknown");
    print_section(
        "MediaWiki",
        &[
            ("Version", core_version_display),
            ("Branch", &config.mw_branch),
            ("Install Path", &config.mw_install_path),
        ]
    );
    println!();

    print_section(
        "Database",
        &[
            ("Type", &config.db_type),
            ("MW Database", &config.mw_database.clone().unwrap_or("Unknown".into())),
        ]
    );
    println!();

    print_section(
        "mwutil",
        &[
            ("Version", &env!("CARGO_PKG_VERSION")),
            ("Debug", &config.debug),
        ]
    );
    Ok(())
}

fn print_section(title: &str, items: &[(&str, &dyn Display)]) {
    println!("{}", title);

    let max_key_len = items.iter()
        .map(|(k, _)| k.len())
        .max()
        .unwrap_or(0);

    for (key, value) in items {
        println!("  {:width$} : {}", key, value, width = max_key_len);
    }
}
