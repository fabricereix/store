use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parser {
    offset: usize,
    buffer: Vec<char>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackageDef {
    pub name: String,
    pub version: String,
    pub url: Option<String>,
    pub build: Option<String>,
}

impl PackageDef {
    pub fn id(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

#[cfg(test)]
pub fn my_package_def() -> PackageDef {
    PackageDef {
        name: "mypackage".to_string(),
        version: "0.1.0".to_string(),
        url: Some("http://localhost:8000/mypackage-0.1.0-x86_64-linux.tar.gz".to_string()),
        build: None,
    }
}

impl Parser {
    pub fn init(s: &str) -> Parser {
        let buffer = s.chars().collect();
        let offset = 0;
        Parser { buffer, offset }
    }

    fn read(&mut self) -> Option<char> {
        match self.buffer.get(self.offset) {
            None => None,
            Some(c) => {
                self.offset += 1;
                Some(*c)
            }
        }
    }
    fn peek(&mut self) -> Option<char> {
        self.buffer.get(self.offset).copied()
    }

    fn remaining(&self) -> String {
        self.buffer.as_slice()[self.offset..].iter().collect()
    }

    pub fn packages(&mut self) -> Result<Vec<PackageDef>, ParseError> {
        let mut packages = vec![];
        let mut existing_packages: HashSet<String> = HashSet::new();
        self.skip_whitespace();
        let mut offset = self.offset;
        while let Some(package) = self.package()? {
            // check that packages are uniquely defined
            let package_id = format!("{}:{}", package.name, package.version);
            if let Some(existing_package) = existing_packages.get(&package_id) {
                let message = format!("Package [{}] has already been defined", existing_package);
                return Err(ParseError { message, offset });
            }
            packages.push(package);
            self.skip_whitespace_or_comment();
            offset = self.offset;
            existing_packages.insert(package_id);
        }
        Ok(packages)
    }

    // make sure that package has at least the url of build field
    fn package(&mut self) -> Result<Option<PackageDef>, ParseError> {
        let offset = self.offset;
        if self.match_literal("[").is_err() {
            Ok(None)
        } else {
            let name = self.package_name()?;
            self.skip_space();
            self.match_literal("@")?;
            self.skip_space();
            let version = self.package_version()?;
            self.skip_space();
            self.match_literal("]")?;
            self.match_newline()?;
            self.skip_whitespace_or_comment();
            let url = self.url_field()?;
            self.skip_whitespace_or_comment();
            let build = self.build()?;
            if url.is_none() && build.is_none() {
                let package_id = format!("{}@{}", name, version);
                let message = format!(
                    "The package [{}] must define at least a url or build field",
                    package_id
                );
                return Err(ParseError { offset, message });
            }
            Ok(Some(PackageDef {
                name,
                version,
                url,
                build,
            }))
        }
    }

    pub fn match_literal(&mut self, s: &str) -> Result<(), ParseError> {
        if self.remaining().starts_with(s) {
            for _ in 0..s.len() {
                self.read();
            }
            Ok(())
        } else {
            let message = format!("Expecting {}", s);
            let offset = self.offset;
            Err(ParseError { message, offset })
        }
    }

    pub fn match_newline(&mut self) -> Result<(), ParseError> {
        if self.match_literal("\n").is_err() {
            let message = "Expecting a newline".to_string();
            let offset = self.offset;
            Err(ParseError { message, offset })
        } else {
            Ok(())
        }
    }

    pub fn whitespace(&mut self) -> String {
        let mut s = "".to_string();
        while let Some(' ') = self.peek() {
            self.read();
            s.push(' ');
        }
        s
    }

    pub fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.read();
            } else {
                break;
            }
        }
    }

    pub fn skip_space(&mut self) {
        while let Some(' ') = self.peek() {
            self.read();
        }
    }

    pub fn skip_whitespace_or_comment(&mut self) {
        loop {
            match self.peek() {
                Some('#') => {
                    self.read();
                    loop {
                        match self.read() {
                            Some('\n') => break,
                            None => break,
                            _ => {}
                        }
                    }
                }
                Some(' ') => {
                    self.read();
                }
                Some('\n') => {
                    self.read();
                }

                _ => break,
            }
        }
    }

    pub fn package_name(&mut self) -> Result<String, ParseError> {
        let mut name = "".to_string();

        loop {
            match self.peek() {
                None => break,
                Some(c) => {
                    if c.is_alphanumeric() || c == '_' || c == '-' {
                        self.read();
                        name.push(c);
                    } else {
                        break;
                    }
                }
            }
        }
        if name.is_empty() {
            let message = "Expecting a package name".to_string();
            let offset = self.offset;
            Err(ParseError { message, offset })
        } else {
            Ok(name)
        }
    }

    pub fn package_version(&mut self) -> Result<String, ParseError> {
        let mut version = "".to_string();

        loop {
            match self.peek() {
                None => break,
                Some(c) => {
                    if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                        self.read();
                        version.push(c);
                    } else {
                        break;
                    }
                }
            }
        }
        if version.is_empty() {
            let message = "Expecting a package version".to_string();
            let offset = self.offset;
            Err(ParseError { message, offset })
        } else {
            Ok(version)
        }
    }

    pub fn url_field(&mut self) -> Result<Option<String>, ParseError> {
        if self.match_literal("url").is_err() {
            Ok(None)
        } else {
            self.skip_space();
            self.match_literal("=")?;
            self.skip_space();
            let url = self.url()?;
            //self.match_newline()?;
            Ok(Some(url))
        }
    }

    pub fn build(&mut self) -> Result<Option<String>, ParseError> {
        if self.match_literal("build").is_err() {
            Ok(None)
        } else {
            let mut commands = "".to_string();
            self.skip_space();
            self.match_literal("=")?;
            self.skip_space();
            let command = self.command()?;
            commands.push_str(command.as_str());
            commands.push('\n');
            self.match_newline()?;
            while let Some(' ') = self.peek() {
                self.skip_whitespace();
                let command = self.command()?;
                commands.push_str(command.as_str());
                commands.push('\n');
                self.match_newline()?;
            }
            Ok(Some(commands))
        }
    }

    pub fn url(&mut self) -> Result<String, ParseError> {
        let mut value = "".to_string();
        //let start = self.offset;
        loop {
            match self.read() {
                None => break,
                Some('\n') => break,
                Some(c) => {
                    value.push(c);
                }
            }
        }
        if value.is_empty() {
            let message = "Expecting an url".to_string();
            let offset = self.offset;
            Err(ParseError { message, offset })
        } else {
            Ok(value.trim().to_string())
        }
    }

    pub fn command(&mut self) -> Result<String, ParseError> {
        let mut value = "".to_string();
        //let start = self.offset;
        loop {
            match self.peek() {
                None => break,
                Some('\n') => break,
                Some(c) => {
                    self.read();
                    value.push(c);
                }
            }
        }
        if value.is_empty() {
            let message = "Expecting a command".to_string();
            let offset = self.offset;
            Err(ParseError { message, offset })
        } else {
            Ok(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::my_package_def;
    use super::*;

    #[test]
    pub fn test_packages() {
        let mut parser = Parser::init(
            r#"[mypackage@0.1.0]
url = http://localhost:8000/mypackage-0.1.0-x86_64-linux.tar.gz
"#,
        );
        assert_eq!(parser.packages().unwrap(), vec![my_package_def()]);
        assert_eq!(parser.offset, 82);
    }

    #[test]
    pub fn test_packages_error_unknown_field() {
        let mut parser = Parser::init(
            r#"
[mypackage@0.1.0]
xxx = yyy
"#,
        );
        assert_eq!(
            parser.packages().err().unwrap(),
            ParseError {
                offset: 1,
                message: "The package [mypackage@0.1.0] must define at least a url or build field"
                    .to_string(),
            }
        );
    }

    #[test]
    pub fn test_packages_error_duplicate_definition() {
        let mut parser = Parser::init(
            r#"
[mypackage@0.1.0]
url = url1
[mypackage@0.1.0]
url = url2
"#,
        );
        assert_eq!(
            parser.packages().err().unwrap(),
            ParseError {
                offset: 30,
                message: "Package [mypackage:0.1.0] has already been defined".to_string(),
            }
        );
    }

    #[test]
    pub fn test_mypackage() {
        let mut parser = Parser::init(
            r#"[mypackage@0.1.0]
# ignore

url = http://localhost:8000/mypackage-0.1.0-x86_64-linux.tar.gz
"#,
        );
        assert_eq!(parser.package().unwrap().unwrap(), my_package_def());
        assert_eq!(parser.offset, 92);
    }

    #[test]
    pub fn test_error_package_missing_url_build() {
        let mut parser = Parser::init(
            r#"[mypackage@0.1.0]
"#,
        );
        assert_eq!(
            parser.package().err().unwrap(),
            ParseError {
                offset: 0,
                message: "The package [mypackage@0.1.0] must define at least a url or build field"
                    .to_string(),
            }
        );
        assert_eq!(parser.offset, 18);
    }

    //
    //     #[test]
    //     pub fn test_url_only() {
    //         let mut parser = Parser::init(r#"[hurl:1.6.0]
    // url: https://github.com/Orange-OpenSource/hurl/releases/download/1.6.0/hurl-1.6.0-x86_64-linux.tar.gz
    // "#);
    //         assert_eq!(
    //             parser.package().unwrap().unwrap(),
    //             package_hurl()
    //         );
    //     }
    //
    //     #[test]
    //     pub fn test_url_plus_build() {
    //         let mut parser = Parser::init(r#"[ruby:2.7.0]
    // url: https://cache.ruby-lang.org/pub/ruby/2.7/ruby-2.7.0.tar.gz
    // build: ./configure --prefix "$PACKAGE_DIR"
    //        make
    //        make install
    // "#);
    //         assert_eq!(
    //             parser.package().unwrap().unwrap(),
    //             package_ruby()
    //         );
    //     }
    //
    //
    //     #[test]
    //     pub fn test_parse() {
    //         let mut parser = Parser::init("[mypackage:0.1.0]\nurl: http://archive\n[mypackage:0.1.0]\n");
    //         assert_eq!(
    //             parser.parse().err().unwrap(),
    //             ParseError { message: "Package <mypackage:0.1.0> has already been defined".to_string(), offset: 38 }
    //         );
    //     }
    //
    //
    //     #[test]
    //     pub fn test_packages() {
    //         let mut parser = Parser::init("[mypackage:0.1.0]\nurl: http://archive\n[mypackage2:0.1.0]\n");
    //         assert_eq!(
    //             parser.packages().unwrap(),
    //             vec![
    //                 PackageDef {
    //                     name: "mypackage".to_string(),
    //                     version: "0.1.0".to_string(),
    //                     url: Some(SourceString {
    //                         value: "http://archive".to_string(),
    //                         span: Span { start: 23, end: 37 },
    //                     }),
    //                     build: None,
    //                 },
    //                 PackageDef {
    //                     name: "mypackage2".to_string(),
    //                     version: "0.1.0".to_string(),
    //                     url: None,
    //                     build: None,
    //                 }
    //             ]
    //         );
    //         assert_eq!(parser.offset, 57);
    //     }
    //
    //
    //     #[test]
    //     pub fn test_package() {
    //         let mut parser = Parser::init("[mypackage:0.1.0]\nurl: http://archive\n[mypackage2:0.1.0]\nbuild: command1\n");
    //         assert_eq!(
    //             parser.package().unwrap().unwrap(),
    //             PackageDef {
    //
    //                 name: "mypackage".to_string(),
    //                 version: "0.1.0".to_string(),
    //                 url: Some(SourceString {
    //                     value: "http://archive".to_string(),
    //                     span: Span { start: 23, end: 37 },
    //
    //                 }),
    //                 build: None,
    //             }
    //         );
    //         assert_eq!(parser.offset, 38);
    //         parser.skip_whitespace();
    //         assert_eq!(
    //             parser.package().unwrap().unwrap(),
    //             PackageDef {
    //                 name: "mypackage2".to_string(),
    //                 version: "0.1.0".to_string(),
    //                 url: None,
    //                 build: Some("command1".to_string()),
    //             }
    //         );
    //         assert_eq!(parser.offset, 73);
    //     }
    //
    //     #[test]
    //     pub fn test_package_name() {
    //         let mut parser = Parser::init("mypackage:0.1.0]");
    //         assert_eq!(
    //             parser.package_name().unwrap(),
    //             "mypackage"
    //         );
    //         assert_eq!(parser.offset, 9);
    //     }
    //
    //     #[test]
    //     pub fn test_match_literal() {
    //         let mut parser = Parser::init(":0.1.0]");
    //         assert!(parser.match_literal(":").is_ok());
    //         assert!(parser.match_literal(":").is_err());
    //     }
    //
    //     #[test]
    //     pub fn test_whitespace() {
    //         let mut parser = Parser::init("a");
    //         assert_eq!(parser.whitespace().as_str(), "");
    //         assert_eq!(parser.offset, 0);
    //
    //         let mut parser = Parser::init(" ");
    //         assert_eq!(parser.whitespace().as_str(), " ");
    //         assert_eq!(parser.offset, 1);
    //     }
    //
    //     #[test]
    //     pub fn test_url_field() {
    //         let mut parser = Parser::init("url:http://archive\n");
    //         assert_eq!(
    //             parser.url_field().unwrap().unwrap(),
    //             SourceString {
    //                 value: "http://archive".to_string(),
    //                 span: Span { start: 4, end: 18 },
    //             }
    //         );
    //         assert_eq!(parser.offset, 19);
    //     }
    //
    //     #[test]
    //     pub fn test_build_field() {
    //         let mut parser = Parser::init(r#"build:command1
    //   command2
    // "#);
    //         assert_eq!(parser.build().unwrap(), vec![
    //             SourceString {
    //                 value: "command1".to_string(),
    //                 span: Span { start: 6, end: 14 },
    //             },
    //             SourceString {
    //                 value: "command2".to_string(),
    //                 span: Span { start: 16, end: 24 },
    //             },
    //         ]);
    //         assert_eq!(parser.offset, 25);
    //
    //         let mut parser = Parser::init(r#"build:command1
    //      command2
    // "#);
    //         assert_eq!(parser.build().unwrap(), vec![
    //             SourceString {
    //                 value: "command1".to_string(),
    //                 span: Span { start: 7, end: 15 },
    //             },
    //             SourceString {
    //                 value: "command2".to_string(),
    //                 span: Span { start: 19, end: 27 },
    //             }
    //         ]);
    //         assert_eq!(parser.offset, 28);
    //     }

    //     #[test]
    //     pub fn test_url_only() {
    //         let mut parser = Parser::init(r#"[hurl:1.6.0]
    // url: https://github.com/Orange-OpenSource/hurl/releases/download/1.6.0/hurl-1.6.0-x86_64-linux.tar.gz
    // "#);
    //         assert_eq!(
    //             parser.package().unwrap().unwrap(),
    //             package_hurl()
    //         );
    //     }
}
