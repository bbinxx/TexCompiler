use std::{collections::BTreeMap, path::PathBuf, time::Duration};
use texcompiler::{
    compiler::{Engine, NativeCompiler, NativeError},
    workspace::Project,
};

fn source(code: &str) -> Project {
    Project::new(
        BTreeMap::from([("main.tex".into(), code.as_bytes().to_vec())]),
        "main.tex".into(),
    )
    .unwrap()
}
fn compiler() -> NativeCompiler {
    NativeCompiler::new(PathBuf::from(env!("CARGO_BIN_EXE_texcompiler-worker")))
}

#[tokio::test]
async fn missing_worker_fails_closed() {
    let error = NativeCompiler::new(PathBuf::from("/texcompiler-missing-worker"))
        .compile(source("plain"), Engine::PdfLatex)
        .await
        .unwrap_err();
    assert!(matches!(error, NativeError::Unavailable));
}

#[tokio::test]
#[ignore = "requires Linux Bubblewrap/prlimit and trusted installed TeX fixtures"]
async fn basic_pdf_on_each_explicit_engine() {
    for engine in [Engine::PdfLatex, Engine::XeLatex, Engine::LuaLatex] {
        let output = compiler()
            .compile(source(include_str!("../fixtures/basic/main.tex")), engine)
            .await
            .unwrap();
        assert!(output.pdf.starts_with(b"%PDF-"));
        assert_eq!(output.page_count, 1);
        assert_eq!(output.engine, engine);
        assert!(!output.engine_version.is_empty());
        let pdf = lopdf::Document::load_mem(&output.pdf).unwrap();
        assert!(
            pdf.extract_text(&[1])
                .unwrap()
                .contains("TexCompiler compatibility baseline.")
        );
    }
}

#[tokio::test]
#[ignore = "requires Linux Bubblewrap/prlimit and trusted installed TeX fixtures"]
async fn multifile_project_compiles_without_source_rewriting() {
    let project = Project::new(
        BTreeMap::from([
            (
                "main.tex".into(),
                include_bytes!("../fixtures/multi-file/main.tex").to_vec(),
            ),
            (
                "body.tex".into(),
                include_bytes!("../fixtures/multi-file/body.tex").to_vec(),
            ),
        ]),
        "main.tex".into(),
    )
    .unwrap();
    let output = compiler().compile(project, Engine::PdfLatex).await.unwrap();
    assert_eq!(output.page_count, 1);
    let pdf = lopdf::Document::load_mem(&output.pdf).unwrap();
    assert!(
        pdf.extract_text(&[1])
            .unwrap()
            .contains("Multi-file compatibility baseline.")
    );
}

#[tokio::test]
#[ignore = "requires Linux Bubblewrap/prlimit; invalid fixture stays inside sandbox"]
async fn nonzero_exit_is_failure_even_after_a_page_was_written() {
    let code =
        r"\documentclass{article}\begin{document}First page\newpage\undefinedcommand\end{document}";
    assert!(matches!(
        compiler().compile(source(code), Engine::PdfLatex).await,
        Err(NativeError::Compilation(_))
    ));
}

#[tokio::test]
#[ignore = "requires Linux Bubblewrap/prlimit; controlled loop runs only inside sandbox"]
async fn wall_timeout_is_enforced() {
    let code = r"\documentclass{article}\begin{document}\loop\iftrue\repeat\end{document}";
    assert!(matches!(
        compiler()
            .with_timeout(Duration::from_millis(500))
            .compile(source(code), Engine::PdfLatex)
            .await,
        Err(NativeError::Timeout)
    ));
}

#[tokio::test]
#[ignore = "requires Linux Bubblewrap/prlimit and installed pdfLaTeX"]
async fn explicit_engine_is_never_silently_changed() {
    let code = r"\documentclass{article}\usepackage{fontspec}\begin{document}Hello\end{document}";
    assert!(matches!(
        compiler().compile(source(code), Engine::PdfLatex).await,
        Err(NativeError::Compilation(_))
    ));
}

#[test]
fn helper_refuses_unsandboxed_startup() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_texcompiler-worker"))
        .env_clear()
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

#[tokio::test]
#[ignore = "requires latexmk plus all three engines in the Linux sandbox"]
async fn latexmk_all_engines_and_nested_main_are_verified() {
    assert!(
        std::path::Path::new("/usr/bin/latexmk").is_file(),
        "install latexmk before this gate"
    );
    for engine in [Engine::PdfLatex, Engine::XeLatex, Engine::LuaLatex] {
        let p = Project::new(
            BTreeMap::from([
                (
                    "chapters/main.tex".into(),
                    include_bytes!("../fixtures/multi-file/main.tex").to_vec(),
                ),
                (
                    "body.tex".into(),
                    include_bytes!("../fixtures/multi-file/body.tex").to_vec(),
                ),
                (
                    ".latexmkrc".into(),
                    b"die 'project rc must never execute';".to_vec(),
                ),
            ]),
            "chapters/main.tex".into(),
        )
        .unwrap();
        let output = compiler().compile(p, engine).await.unwrap();
        assert_eq!(output.method, "latexmk");
        assert_eq!(output.page_count, 1);
        assert_eq!(output.engine, engine);
    }
}

#[tokio::test]
#[ignore = "requires Linux Bubblewrap and installed pdfLaTeX"]
async fn multipass_resolves_cross_references() {
    let code = r"\documentclass{article}\begin{document}Reference: \ref{target}.\newpage\section{Target}\label{target}\end{document}";
    let output = compiler()
        .compile(source(code), Engine::PdfLatex)
        .await
        .unwrap();
    assert_eq!(output.page_count, 2);
    assert!(
        !output
            .warnings
            .iter()
            .any(|line| line.contains("undefined"))
    );
}

#[tokio::test]
#[ignore = "requires Linux sandbox with XeLaTeX and LuaLaTeX fonts"]
async fn unicode_fontspec_source_compiles_on_unicode_engines() {
    let code = r"\documentclass{article}\usepackage{fontspec}\setmainfont{lmroman10-regular.otf}\begin{document}Unicode: café Ω\end{document}";
    for engine in [Engine::XeLatex, Engine::LuaLatex] {
        let output = compiler().compile(source(code), engine).await.unwrap();
        assert_eq!(output.page_count, 1);
        let pdf = lopdf::Document::load_mem(&output.pdf).unwrap();
        assert!(pdf.extract_text(&[1]).unwrap().contains("café"));
    }
}

#[tokio::test]
#[ignore = "requires Linux sandbox and all three TeX engines"]
async fn shell_escape_is_effectively_disabled_for_each_engine() {
    let code = r"\documentclass{article}\usepackage{shellesc}\begin{document}\ifnum\ShellEscapeStatus=0 Disabled\else\errmessage{shell escape enabled}\fi\end{document}";
    for engine in [Engine::PdfLatex, Engine::XeLatex, Engine::LuaLatex] {
        let output = compiler().compile(source(code), engine).await.unwrap();
        assert_eq!(output.page_count, 1);
    }
}
