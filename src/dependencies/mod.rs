use crate::PackageDef;

type DependencyError = String;

// Resolve dependencies
// the dependency must exist and be  unique (one unique version)
// there must be no cycle
pub fn resolve(
    packages_defs: &Vec<PackageDef>,
) -> Result<Vec<(String, PackageDef)>, DependencyError> {
    let mut dependencies: Vec<(String, PackageDef)> = vec![];
    for package_def in packages_defs {
        for query in package_def.depends.clone() {
            let p = find_package(packages_defs, &query)?;
            dependencies.push((package_def.id(), p))
        }
    }

    Ok(dependencies)
}

fn find_package(package_defs: &[PackageDef], query: &str) -> Result<PackageDef, DependencyError> {
    let packages = package_defs
        .iter()
        .filter(|p| {
            query == p.name.as_str() || query == format!("{}@{}", p.name, p.version).as_str()
        })
        .cloned()
        .collect::<Vec<PackageDef>>();

    if packages.is_empty() {
        Err(format!("Package dependency {} can not be found", query))
    } else if packages.len() > 1 {
        Err(format!(
            "Package dependency {} can not be uniquely resolved",
            query
        ))
    } else {
        Ok(packages.get(0).unwrap().clone())
    }
}
