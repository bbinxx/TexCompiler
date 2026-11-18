//! Conservative recommendations; explicit compile selections bypass this code.
use super::Engine;
use regex::Regex;
use std::sync::OnceLock;

pub fn detect_engine(code: &str, preferred: Engine) -> Engine {
    let mut clean = String::new();
    for line in code.lines() {
        let mut slashes = 0;
        for ch in line.chars() {
            if ch == '%' && slashes % 2 == 0 {
                break;
            }
            clean.push(ch);
            slashes = if ch == '\\' { slashes + 1 } else { 0 };
        }
        clean.push('\n');
    }
    static PACKAGES: OnceLock<Regex> = OnceLock::new();
    let packages = PACKAGES.get_or_init(|| {
        Regex::new(r"\\(?:usepackage|RequirePackage)\s*(?:\[[^\]]*\]\s*)?\{([^}]*)\}")
            .expect("constant package regex")
    });
    let mut lua = false;
    let mut xe = false;
    let mut unicode = false;
    for capture in packages.captures_iter(&clean) {
        for package in capture[1].split(',').map(str::trim) {
            lua |= matches!(
                package,
                "luacode" | "luacolor" | "luamplib" | "luatexja" | "luabidi" | "luaplot"
            );
            xe |= matches!(package, "xeCJK" | "xepersian");
            unicode |= package == "fontspec";
        }
    }
    unicode |= [
        "\\setmainfont",
        "\\setsansfont",
        "\\setmonofont",
        "\\newfontfamily",
    ]
    .iter()
    .any(|command| clean.contains(command));
    if lua {
        Engine::LuaLatex
    } else if xe {
        Engine::XeLatex
    } else if unicode && preferred == Engine::PdfLatex {
        Engine::LuaLatex
    } else {
        preferred
    }
}
