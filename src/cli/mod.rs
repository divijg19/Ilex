use std::env;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryAlias {
    Corefetch,
    Core,
    Cf,
    Ilex,
    Unknown(String),
}

impl BinaryAlias {
    fn from_program_name(name: &str) -> Self {
        match name {
            "corefetch" => Self::Corefetch,
            "core" => Self::Core,
            "cf" => Self::Cf,
            "ilex" => Self::Ilex,
            other => Self::Unknown(other.to_owned()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Corefetch => "corefetch",
            Self::Core => "core",
            Self::Cf => "cf",
            Self::Ilex => "ilex",
            Self::Unknown(name) => name.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    binary_name: String,
    alias: BinaryAlias,
    raw_args: Vec<String>,
}

impl Invocation {
    pub fn from_env() -> Self {
        let raw_args: Vec<String> = env::args().collect();
        let binary_name = raw_args
            .first()
            .map(|value| program_name(value))
            .unwrap_or_else(|| "corefetch".to_owned());
        let alias = BinaryAlias::from_program_name(binary_name.as_str());
        let user_args = raw_args.into_iter().skip(1).collect();

        Self {
            binary_name,
            alias,
            raw_args: user_args,
        }
    }

    pub fn binary_name(&self) -> &str {
        &self.binary_name
    }

    pub fn alias_name(&self) -> &str {
        self.alias.as_str()
    }

    pub fn user_args(&self) -> &[String] {
        &self.raw_args
    }
}

fn program_name(value: &str) -> String {
    Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("corefetch")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{BinaryAlias, program_name};

    #[test]
    fn detects_known_aliases() {
        assert_eq!(BinaryAlias::from_program_name("core").as_str(), "core");
        assert_eq!(BinaryAlias::from_program_name("cf").as_str(), "cf");
        assert_eq!(BinaryAlias::from_program_name("ilex").as_str(), "ilex");
    }

    #[test]
    fn extracts_binary_name_from_path() {
        assert_eq!(program_name("/tmp/bin/corefetch"), "corefetch");
    }
}
