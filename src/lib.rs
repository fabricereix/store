mod compiler;
mod core;
mod dependencies;
mod installer;
mod parser;

pub use self::compiler::*;
pub use self::core::*;
pub use self::dependencies::resolve as resolve_dependencies;
pub use self::installer::*;
pub use self::parser::*;

#[cfg(test)]
pub use self::parser::my_package_def;
