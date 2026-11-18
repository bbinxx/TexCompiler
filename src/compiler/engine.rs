use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    PdfLatex,
    XeLatex,
    LuaLatex,
}

#[derive(Debug, Error)]
#[error("unsupported engine; use pdflatex, xelatex or lualatex")]
pub struct UnsupportedEngine;

impl FromStr for Engine {
    type Err = UnsupportedEngine;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pdflatex" => Ok(Self::PdfLatex),
            "xelatex" => Ok(Self::XeLatex),
            "lualatex" => Ok(Self::LuaLatex),
            _ => Err(UnsupportedEngine),
        }
    }
}

impl Engine {
    pub fn executable(self) -> &'static str {
        match self {
            Self::PdfLatex => "/usr/bin/pdflatex",
            Self::XeLatex => "/usr/bin/xelatex",
            Self::LuaLatex => "/usr/bin/lualatex",
        }
    }

    /// Fixed compiler arguments; callers must pass a validated main path.
    pub fn arguments(self, main: &str, latexmk: bool) -> Vec<String> {
        let mut args = Vec::new();
        if latexmk {
            args.push("-norc".into());
            args.push(
                match self {
                    Self::PdfLatex => "-pdf",
                    Self::XeLatex => "-xelatex",
                    Self::LuaLatex => "-lualatex",
                }
                .into(),
            );
        }
        args.extend(
            [
                "-no-shell-escape",
                "-interaction=nonstopmode",
                "-halt-on-error",
                "-jobname=document",
                "-output-directory=/output",
            ]
            .map(str::to_string),
        );
        args.push(format!("./{main}"));
        args
    }
}
