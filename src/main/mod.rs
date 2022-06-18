extern crate fs_extra;
extern crate humansize;
extern crate store;

mod exit;
mod options;
mod pos;
mod utils;

use exit::*;
use humansize::FileSize;
use options::*;
use pos::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use store::{resolve_dependencies, Package, PackageDef};
use utils::dir_size;

fn main() {
    let options = match parse_options() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{}", message);
            ExitCode::ErrorOptions.exit()
        }
    };

    if options.verbose {
        eprintln!("{:#?}", options);
    }
    let package_defs = parse_database_file(&options.db_file);
    let dependencies = resolve_deps(&package_defs);

    match options.command.clone() {
        Command::Install(package_queries) => {
            let packages = find_packages(package_defs, &package_queries);
            let install_packages = resolve_packages(&packages, &dependencies);
            for package_def in install_packages {
                let package = package_def.compile(); // can not fail
                install(
                    &package,
                    &options.packages_dir,
                    &options.tmp_dir,
                    options.verbose,
                );
            }
            ExitCode::Success.exit()
        }
        Command::ReInstall(package_queries) => {
            let install_packages = find_packages(package_defs, &package_queries);
            for package_def in install_packages {
                let package = package_def.compile(); // can not fail
                delete_package(&package, &options.packages_dir, options.verbose);
                install(
                    &package,
                    &options.packages_dir,
                    &options.tmp_dir,
                    options.verbose,
                );
            }
            ExitCode::Success.exit()
        }
        Command::UnInstall(package_queries) => {
            let install_packages = find_packages(package_defs, &package_queries);
            for package_def in install_packages {
                let package = package_def.compile(); // can not fail
                delete_package(&package, &options.packages_dir, options.verbose);
            }
            ExitCode::Success.exit()
        }
        Command::Info => info(package_defs, &options.packages_dir),
        Command::Dependencies => display_dependencies(&dependencies),
    }
}

fn parse_database_file(db_file: &Path) -> Vec<PackageDef> {
    let mut file = File::open(db_file).expect("file exists");
    let mut content = String::new();
    if file.read_to_string(&mut content).is_err() {
        eprintln!("Can not read databse file {}", db_file.display());
        ExitCode::ErrorReadingDatabaseFile.exit();
    } else {
        let mut db_parser = store::Parser::init(&content);
        match db_parser.packages() {
            Ok(packages) => packages,
            Err(e) => {
                let pos = Pos::find(&content, e.offset);
                eprintln!(
                    "Parsing Error at {}:{}:{}",
                    db_file.display(),
                    pos.line,
                    pos.column
                );
                eprintln!("{}", e.message);
                ExitCode::ErrorParse.exit()
            }
        }
    }
}

fn resolve_deps(package_defs: &Vec<PackageDef>) -> Vec<(String, PackageDef)> {
    match resolve_dependencies(package_defs) {
        Ok(deps) => deps,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::ErrorDependencies.exit()
        }
    }
}

fn find_packages(package_defs: Vec<PackageDef>, package_queries: &Vec<String>) -> Vec<PackageDef> {
    let mut install_packages = vec![];

    for package_query in package_queries {
        let packages = package_defs
            .iter()
            .filter(|p| {
                package_query == p.name.as_str()
                    || package_query == format!("{}@{}", p.name, p.version).as_str()
            })
            .cloned()
            .collect::<Vec<PackageDef>>();

        if packages.is_empty() {
            eprintln!("Package {} is not defined", package_query);
            ExitCode::ErrorPackageNotDefined.exit()
        } else {
            for package in packages {
                install_packages.push(package);
            }
        }
    }
    install_packages
}

// add dependent packages to install
fn resolve_packages(
    package_defs: &Vec<PackageDef>,
    dependencies: &Vec<(String, PackageDef)>,
) -> Vec<PackageDef> {
    let mut install_packages = vec![];

    for package in package_defs {
        for dep in dependencies {
            if dep.0 == package.id() && !install_packages.contains(&dep.1) {
                install_packages.push(dep.1.clone());
            }
        }
        if !install_packages.contains(package) {
            install_packages.push(package.clone());
        }
    }
    install_packages
}
//
// fn compile(package_def: &PackageDef) -> Package {
//     match package_def.compile() {
//         Ok(package) => package,
//         Err(e) => {
//             dbg!(e);
//             ExitCode::ErrorCompile.exit()
//         }
//     }
// }

fn delete_package(package: &Package, packages_dir: &Path, _verbose: bool) {
    let package_dir = packages_dir.join(package.id.clone());
    if package_dir.exists() {
        match fs::remove_dir_all(package_dir.display().to_string()) {
            Ok(_) => {
                eprintln!("Directory {} has been deleted", package_dir.display());
            }
            Err(e) => {
                eprintln!("error deleting {} - {}", package_dir.display(), e);
                ExitCode::ErrorInstall.exit();
            }
        };
    }
}

fn install(package: &Package, packages_dir: &Path, tmp_dir: &Path, verbose: bool) {
    if verbose {
        eprintln!("Installing {}", package.id);
    }
    let mut package_installer = match store::Installer::init(packages_dir, tmp_dir, package) {
        Ok(inst) => inst,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::ErrorInstall.exit();
        }
    };

    if package_installer.is_installed() {
        println!("Package {} already installed", package.id);
        return;
    }

    match package_installer.create_directory() {
        Ok(message) => println!("{}", message),
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::ErrorInstall.exit();
        }
    }
    if let Err(e) = package_installer.create_directory() {
        eprintln!("{}", e);
        ExitCode::ErrorInstall.exit();
    }

    for command in &package.commands {
        if verbose {
            eprintln!("Executing {:?}", command);
        }
        match package_installer.exec_command(command, verbose) {
            Ok(message) => {
                if !message.is_empty() {
                    println!("{}", message);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                let message = package_installer.delete_directory();
                println!("{}", message);
                ExitCode::ErrorInstall.exit();
            }
        }
    }
    println!("Package {} successfully installed", package.id);
}

// display size of the installed package or - (if not installed)
// add * if not defined
fn info(package_defs: Vec<PackageDef>, packages_dir: &Path) {
    let mut defined_packages = HashSet::new();
    for package in package_defs {
        defined_packages.insert(package.id());
    }

    let mut installed_packages: HashMap<String, u64> = HashMap::new();
    let dir_entries = match fs::read_dir(packages_dir) {
        Ok(paths) => paths,
        Err(_) => {
            eprintln!("Can not read {}", packages_dir.display());
            ExitCode::ErrorInfo.exit()
        }
    };
    for dir_entry in dir_entries {
        let path = dir_entry.unwrap().path();
        let package_name = path.clone();
        let package_name = package_name.file_name();
        let package_name = package_name.unwrap().to_str().unwrap().to_string();

        // let size = match fs_extra::dir::get_size(path.clone()) {
        //     Ok(s) => s,
        //     Err(e) => {
        //         eprintln!(
        //             "Error calculating size for folder {}: {}",
        //             path.display(),
        //             e
        //         );
        //         0
        //     }
        // };
        let size = match dir_size(path.clone()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "Error calculating size for folder {}: {}",
                    path.display(),
                    e
                );
                0
            }
        };
        installed_packages.insert(package_name, size);
    }

    let mut packages = HashSet::new();
    for package in &defined_packages {
        packages.insert(package.to_string());
    }
    for package in installed_packages.keys().cloned().collect::<Vec<String>>() {
        packages.insert(package.to_string());
    }

    let mut packages = packages.iter().cloned().collect::<Vec<String>>();
    packages.sort();

    let mut name_column_length = 0;
    for package in packages.clone() {
        let tokens = package.split('@').collect::<Vec<&str>>();
        let name = tokens.get(0).unwrap();
        if name.len() > name_column_length {
            name_column_length = name.len();
        }
    }
    eprintln!(
        "{name:width$}{version:12}{size:10}",
        width = name_column_length + 1,
        name = "Name",
        version = "Version",
        size = "Size"
    );
    eprintln!("==============================================================");

    for package in packages {
        let size = match installed_packages.get(&package) {
            None => "-".to_string(),
            Some(size) => size
                .file_size(humansize::file_size_opts::CONVENTIONAL)
                .unwrap(),
        };
        let obsolete = if defined_packages.contains(&package) {
            "".to_string()
        } else {
            "obsolete".to_string()
        };
        let tokens = package.split('@').collect::<Vec<&str>>();
        let name = tokens.get(0).unwrap();
        let version = match tokens.get(1) {
            None => "",
            Some(v) => v,
        };
        println!(
            "{name:width$}{version:12}{size:10}{obsolete}",
            width = name_column_length + 1,
            name = name,
            version = version,
            size = size,
            obsolete = obsolete
        );
    }
}

fn display_dependencies(dependencies: &Vec<(String, PackageDef)>) {
    println!("Dependencies");
    for dep in dependencies {
        println!("{} -> {}", dep.0, dep.1.id())
    }
}
