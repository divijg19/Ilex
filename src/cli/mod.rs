use std::env;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Fetch,
    Minimal,
    Json,
}

impl OutputMode {
    pub fn renderer_key(&self) -> &'static str {
        match self {
            Self::Fetch => "fetch-text",
            Self::Minimal => "minimal-text",
            Self::Json => "json",
        }
    }
}

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

    pub fn canonical_command(&self) -> &'static str {
        "corefetch"
    }

    pub fn is_primary(&self) -> bool {
        matches!(self, Self::Corefetch)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    binary_name: String,
    alias: BinaryAlias,
    output_mode: OutputMode,
    raw_args: Vec<String>,
}

impl Invocation {
    pub fn from_env() -> Self {
        Self::from_args(env::args())
    }

    fn from_args(args: impl IntoIterator<Item = String>) -> Self {
        let raw_args: Vec<String> = args.into_iter().collect();
        let binary_name = raw_args
            .first()
            .map(|value| program_name(value))
            .unwrap_or_else(|| "corefetch".to_owned());
        let alias = BinaryAlias::from_program_name(binary_name.as_str());
        let mut output_mode = OutputMode::Fetch;
        let mut user_args = Vec::new();

        for argument in raw_args.into_iter().skip(1) {
            match argument.as_str() {
                "--json" => output_mode = OutputMode::Json,
                "--minimal" => output_mode = OutputMode::Minimal,
                _ => user_args.push(argument),
            }
        }

        Self {
            binary_name,
            alias,
            output_mode,
            raw_args: user_args,
        }
    }

    pub fn binary_name(&self) -> &str {
        &self.binary_name
    }

    pub fn alias_name(&self) -> &str {
        self.alias.as_str()
    }

    pub fn canonical_command(&self) -> &'static str {
        self.alias.canonical_command()
    }

    pub fn is_primary_entrypoint(&self) -> bool {
        self.alias.is_primary()
    }

    pub fn output_mode(&self) -> OutputMode {
        self.output_mode
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
    use super::{BinaryAlias, Invocation, OutputMode, program_name};

    #[test]
    fn detects_known_aliases() {
        assert_eq!(
            BinaryAlias::from_program_name("corefetch").as_str(),
            "corefetch"
        );
        assert_eq!(BinaryAlias::from_program_name("core").as_str(), "core");
        assert_eq!(BinaryAlias::from_program_name("cf").as_str(), "cf");
        assert_eq!(BinaryAlias::from_program_name("ilex").as_str(), "ilex");
    }

    #[test]
    fn preserves_corefetch_as_primary_command() {
        let alias = BinaryAlias::from_program_name("corefetch");

        assert!(alias.is_primary());
        assert_eq!(alias.canonical_command(), "corefetch");
    }

    #[test]
    fn extracts_binary_name_from_path() {
        assert_eq!(program_name("/tmp/bin/corefetch"), "corefetch");
    }

    #[test]
    fn parses_json_output_mode() {
        let invocation = Invocation::from_args(vec![
            "corefetch".to_owned(),
            "--json".to_owned(),
            "--verbose".to_owned(),
        ]);

        assert_eq!(invocation.output_mode(), OutputMode::Json);
        assert_eq!(invocation.user_args(), &["--verbose".to_owned()]);
    }

    #[test]
    fn later_output_flag_overrides_earlier_flag() {
        let invocation = Invocation::from_args(vec![
            "corefetch".to_owned(),
            "--minimal".to_owned(),
            "--json".to_owned(),
        ]);

        assert_eq!(invocation.output_mode(), OutputMode::Json);
    }
}
