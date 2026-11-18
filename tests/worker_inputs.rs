use std::collections::BTreeMap;
use texcompiler::{compiler::Engine, workspace::Project};

fn project(
    files: &[(&str, &[u8])],
    main: &str,
) -> Result<Project, texcompiler::workspace::InputError> {
    Project::new(
        files
            .iter()
            .map(|(name, bytes)| (name.to_string(), bytes.to_vec()))
            .collect(),
        main.into(),
    )
}

#[test]
fn preserves_asset_bytes_and_project_relative_paths() {
    let p = project(
        &[
            ("main.tex", b"source"),
            ("fonts/My Font.otf", &[0, 1, 255]),
            ("parts/body.tex", b"body"),
        ],
        "main.tex",
    )
    .unwrap();
    let workspace = p.prepare().unwrap();
    assert_eq!(
        std::fs::read(workspace.path().join("fonts/My Font.otf")).unwrap(),
        [0, 1, 255]
    );
    assert_eq!(
        std::fs::read(workspace.path().join("parts/body.tex")).unwrap(),
        b"body"
    );
    let path = workspace.path().to_owned();
    drop(workspace);
    assert!(!path.exists());
}

#[test]
fn rejects_nonportable_and_option_like_paths_before_writing() {
    for name in [
        "",
        "/absolute.tex",
        "../escape.tex",
        "a/../b.tex",
        "./main.tex",
        "a//b.tex",
        "a/./b.tex",
        "C:/main.tex",
        "C:\\main.tex",
        "\\\\server\\share",
        "-main.tex",
        "a/-x.tex",
        "a\n.tex",
        "a\0.tex",
        "trailing/",
    ] {
        assert!(
            project(&[("main.tex", b"source"), (name, b"bad")], "main.tex").is_err(),
            "{name:?}"
        );
    }
}

#[test]
fn rejects_missing_or_unsafe_main_and_file_directory_collisions() {
    for main in ["missing.tex", "../main.tex", "main.pdf"] {
        assert!(project(&[("main.tex", b"source")], main).is_err());
    }
    assert!(
        project(
            &[
                ("main.tex", b"source"),
                ("assets", b"file"),
                ("assets/a.png", b"image")
            ],
            "main.tex"
        )
        .is_err()
    );
    assert!(
        project(
            &[("main.tex", b"source"), ("main.tex/body", b"x")],
            "main.tex"
        )
        .is_err()
    );
}

#[test]
fn enforces_project_count_and_byte_limits() {
    let mut files = BTreeMap::from([("main.tex".into(), b"source".to_vec())]);
    for n in 0..256 {
        files.insert(format!("asset{n}"), vec![]);
    }
    assert!(Project::new(files, "main.tex".into()).is_err());
    assert!(
        Project::new(
            BTreeMap::from([("main.tex".into(), vec![0; 20 * 1024 * 1024 + 1])]),
            "main.tex".into()
        )
        .is_err()
    );
}

#[test]
fn engines_are_allowlisted_and_commands_disable_external_execution() {
    for (name, expected) in [
        ("pdflatex", Engine::PdfLatex),
        ("xelatex", Engine::XeLatex),
        ("lualatex", Engine::LuaLatex),
    ] {
        assert_eq!(name.parse::<Engine>().unwrap(), expected);
        let args = expected.arguments("main.tex", false);
        assert!(args.iter().any(|a| a == "-no-shell-escape"));
        assert_eq!(args.last().unwrap(), "./main.tex");
        let args = expected.arguments("main.tex", true);
        let flag = match name {
            "pdflatex" => "-pdf",
            "xelatex" => "-xelatex",
            "lualatex" => "-lualatex",
            _ => unreachable!(),
        };
        assert!(args.iter().any(|arg| arg == flag));
        assert_eq!(args.first().unwrap(), "-norc");
        assert!(args.iter().any(|a| a == "-no-shell-escape"));
    }
    for name in [
        "sh",
        "/usr/bin/pdflatex",
        "pdflatex --shell-escape",
        "auto",
        "",
    ] {
        assert!(name.parse::<Engine>().is_err());
    }
}
