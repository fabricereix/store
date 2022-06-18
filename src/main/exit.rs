pub enum ExitCode {
    Success,
    ErrorOptions,
    ErrorReadingDatabaseFile,
    ErrorParse,
    ErrorDependencies,
    ErrorPackageNotDefined,
    ErrorInstall,
    ErrorInfo,
}

impl ExitCode {
    pub fn exit(self) -> ! {
        let value = match self {
            ExitCode::Success => 0,
            // exit code 2 is reserved for command-line options parsing
            // used by default by clap
            ExitCode::ErrorOptions => 2,
            ExitCode::ErrorReadingDatabaseFile => 3,
            ExitCode::ErrorParse => 4,
            ExitCode::ErrorDependencies => 4,
            ExitCode::ErrorPackageNotDefined => 5,
            ExitCode::ErrorInstall => 8,
            ExitCode::ErrorInfo => 9,
        };
        std::process::exit(value)
    }
}
