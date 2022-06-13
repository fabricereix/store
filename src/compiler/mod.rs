use super::{Command, ExtractCommand, Package, PackageDef};

#[derive(Clone, Debug, PartialEq)]
pub struct CompileError {
    pub offset: usize,
    pub message: String,
}

impl PackageDef {
    pub fn compile(&self) -> Package {
        let mut commands = vec![];
        if let Some(url) = self.url.clone() {
            let command = Command::Download(url.clone());
            commands.push(command);
            if let Some(extract_command) = ExtractCommand::get(&url) {
                commands.push(extract_command);
            }
        }
        if let Some(build) = self.build.clone() {
            let command = Command::Shell(build);
            commands.push(command);
        }

        if self.build.is_none() {
            let command = Command::Copy();
            commands.push(command);
        }
        let id = format!("{}@{}", self.name, self.version);
        Package { id, commands }
    }
}

impl ExtractCommand {
    fn get(url: &str) -> Option<Command> {
        if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
            Some(Command::Extract(ExtractCommand::TarGz))
        } else if url.ends_with(".tar.xz") {
            Some(Command::Extract(ExtractCommand::TarXz))
        } else if url.ends_with(".tar.bz2") {
            Some(Command::Extract(ExtractCommand::TarBz2))
        } else if url.ends_with(".zip") {
            Some(Command::Extract(ExtractCommand::Zip))
        } else {
            None
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::super::{my_package_def, mypackage};

    #[test]
    pub fn test_url_only() {
        assert_eq!(my_package_def().compile(), mypackage())
    }
    //
    // #[test]
    // pub fn test_url_plus_build() {
    //     assert_eq!(
    //         package_ruby().compile(),
    //         vec![
    //             Command {
    //                 span: Span { start: 18, end: 76 },
    //                 kind: CommandKind::Download("https://cache.ruby-lang.org/pub/ruby/2.7/ruby-2.7.0.tar.gz".to_string()),
    //             },
    //             Command {
    //                 span: Span { start: 18, end: 76 },
    //                 kind: CommandKind::Extract(ExtractCommand::TarGz),
    //             },
    //             Command {
    //                 span: Span { start: 84, end: 119 },
    //                 kind: CommandKind::Shell(r#"./configure --prefix "$PACKAGE_DIR""#.to_string()),
    //             },
    //             Command {
    //                 span: Span { start: 127, end: 131 },
    //                 kind: CommandKind::Shell(r#"make"#.to_string()),
    //             },
    //             Command {
    //                 span: Span { start: 139, end: 151 },
    //                 kind: CommandKind::Shell(r#"make install"#.to_string()),
    //             }
    //         ]
    //     )
    // }
}
