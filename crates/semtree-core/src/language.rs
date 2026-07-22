use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Tsx,
    Go,
    Java,
    C,
    Cpp,
    CSharp,
    Ruby,
    Php,
    Kotlin,
    Scala,
    Swift,
    OCaml,
    Solidity,
    Lua,
    Zig,
    Elisp,
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs" => Self::Rust,
            "py" => Self::Python,
            "js" | "mjs" | "cjs" | "jsx" => Self::JavaScript,
            "ts" => Self::TypeScript,
            "tsx" => Self::Tsx,
            "go" => Self::Go,
            "java" => Self::Java,
            "c" | "h" => Self::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Self::Cpp,
            "cs" => Self::CSharp,
            "rb" => Self::Ruby,
            "php" => Self::Php,
            "kt" | "kts" => Self::Kotlin,
            "scala" | "sc" => Self::Scala,
            "swift" => Self::Swift,
            "ml" | "mli" => Self::OCaml,
            "sol" => Self::Solidity,
            "lua" => Self::Lua,
            "zig" => Self::Zig,
            "el" => Self::Elisp,
            _ => Self::Unknown,
        }
    }

    pub fn from_path(path: &std::path::Path) -> Self {
        path.extension()
            .and_then(|e| e.to_str())
            .map(Self::from_extension)
            .unwrap_or(Self::Unknown)
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
            Self::Go => "go",
            Self::Java => "java",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::CSharp => "csharp",
            Self::Ruby => "ruby",
            Self::Php => "php",
            Self::Kotlin => "kotlin",
            Self::Scala => "scala",
            Self::Swift => "swift",
            Self::OCaml => "ocaml",
            Self::Solidity => "solidity",
            Self::Lua => "lua",
            Self::Zig => "zig",
            Self::Elisp => "elisp",
            Self::Unknown => "unknown",
        };
        write!(f, "{s}")
    }
}
