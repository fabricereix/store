use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub command: Command,
    pub db_file: PathBuf,
    pub packages_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub verbose: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Install(Vec<String>),
    ReInstall(Vec<String>),
    UnInstall(Vec<String>),
    Info,
}

// clap (unfortunately) panics when options are not good
// for consistency, you should exit in case of errors.
// But I would have prefer the standard Result return type!
pub fn parse_options() -> Result<Options, String> {
    let command = clap::Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            clap::Arg::new("db_file")
                .long("db-file")
                .takes_value(true)
                .help("Specify database file (default is config/db.ini)"),
        )
        .arg(
            clap::Arg::new("tmp_dir")
                .long("tmp-dir")
                .takes_value(true)
                .help("Specify store tmp directory (default is ./tmp)"),
        )
        .arg(
            clap::Arg::new("packages_dir")
                .long("packages-dir")
                .takes_value(true)
                .help("Specify packages directory (default is ./packages)"),
        )
        .arg(
            clap::Arg::new("verbose")
                .long("verbose")
                .help("Turn verbose mode"),
        )
        .subcommand(
            clap::Command::new("install")
                .about("Install a specific package")
                .arg(
                    clap::Arg::new("package_queries")
                        .help("Package to be installed: NAME or NAME:VERSION")
                        .multiple_occurrences(true)
                        .required(true),
                ),
        )
        .subcommand(
            clap::Command::new("reinstall")
                .about("Re-Install a specific package")
                .arg(
                    clap::Arg::new("package_queries")
                        .help("Package to be reinstalled: NAME or NAME:VERSION")
                        .multiple_occurrences(true)
                        .required(true),
                ),
        )
        .subcommand(
            clap::Command::new("uninstall")
                .about("Uninstall a specific package")
                .arg(
                    clap::Arg::new("package_queries")
                        .help("Package to be uninstall: NAME or NAME:VERSION")
                        .multiple_occurrences(true)
                        .required(true),
                ),
        )
        .subcommand(clap::Command::new("info").about("List packages (installed and/or defined)"));
    let matches = command.clone().get_matches();

    let command = if let Some(("install", install_options)) = matches.subcommand() {
        let package_queries = install_options
            .values_of("package_queries")
            .expect("package_queries");
        Command::Install(package_queries.map(|s| s.to_string()).collect())
    } else if let Some(("reinstall", install_options)) = matches.subcommand() {
        let package_queries = install_options
            .values_of("package_queries")
            .expect("package_queries");
        Command::ReInstall(package_queries.map(|s| s.to_string()).collect())
    } else if let Some(("uninstall", install_options)) = matches.subcommand() {
        let package_queries = install_options
            .values_of("package_queries")
            .expect("package_queries");
        Command::UnInstall(package_queries.map(|s| s.to_string()).collect())
    } else if let Some(("info", _)) = matches.subcommand() {
        Command::Info
    } else {
        command.clone().print_help().unwrap();
        std::process::exit(2);
    };

    let db_file = get_db_file(matches.value_of("db_file"))?;
    let tmp_dir = get_tmp_dir(matches.value_of("tmp_dir"))?;
    let packages_dir = get_packages_dir(matches.value_of("packages_dir"))?;
    let verbose = matches.is_present("verbose");
    Ok(Options {
        command,
        db_file,
        tmp_dir,
        packages_dir,
        verbose,
    })
}

fn get_db_file(value: Option<&str>) -> Result<PathBuf, String> {
    let path = match value {
        None => match std::env::var("STORE_DB_FILE") {
            Ok(value) => value,
            Err(_) => "config/db.ini".to_string(),
        },
        Some(s) => s.to_string(),
    };
    let path = Path::new(&path);
    if !path.exists() {
        return Err(format!("db_file {} does not exist!", path.display()));
    }
    Ok(path.to_path_buf())
}

fn get_packages_dir(value: Option<&str>) -> Result<PathBuf, String> {
    let path = match value {
        None => match std::env::var("STORE_PACKAGES_DIR") {
            Ok(value) => value,
            Err(_) => "/store".to_string(),
        },
        Some(s) => s.to_string(),
    };
    let path = Path::new(&path);
    Ok(path.to_path_buf())
}

fn get_tmp_dir(value: Option<&str>) -> Result<PathBuf, String> {
    let path = match value {
        None => match std::env::var("STORE_TMP_DIR") {
            Ok(value) => value,
            Err(_) => "/tmp/store".to_string(),
        },
        Some(s) => s.to_string(),
    };
    let path = Path::new(&path);
    Ok(path.to_path_buf())
}
