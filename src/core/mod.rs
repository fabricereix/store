#[derive(Clone, Debug, PartialEq)]
pub struct Package {
    pub id: String,
    pub commands: Vec<Command>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Download(String),
    Extract(ExtractCommand),
    Copy(), // "hardcopy" copy file from current directory to package directory
    Shell(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExtractCommand {
    TarGz,
    TarXz,
    TarBz2,
    Zip,
}

#[cfg(test)]
pub fn mypackage() -> Package {
    Package {
        id: "mypackage@0.1.0".to_string(),
        commands: vec![
            Command::Download(
                "http://localhost:8000/mypackage-0.1.0-x86_64-linux.tar.gz".to_string(),
            ),
            Command::Extract(ExtractCommand::TarGz),
            Command::Copy(),
        ],
    }
}
