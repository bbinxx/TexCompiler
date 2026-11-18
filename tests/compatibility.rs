use std::{collections::BTreeMap, path::PathBuf, process::Command};
use texcompiler::{
    compiler::{Engine, NativeCompiler, NativeError},
    workspace::Project,
};

fn compiler() -> NativeCompiler {
    NativeCompiler::new(PathBuf::from(env!("CARGO_BIN_EXE_texcompiler-worker")))
}
async fn compile(
    files: BTreeMap<String, Vec<u8>>,
    engine: Engine,
) -> texcompiler::compiler::Compilation {
    compiler()
        .compile(Project::new(files, "main.tex".into()).unwrap(), engine)
        .await
        .unwrap()
}
fn text(source: &str) -> BTreeMap<String, Vec<u8>> {
    BTreeMap::from([("main.tex".into(), source.as_bytes().to_vec())])
}
fn font(name: &str) -> Vec<u8> {
    let output = Command::new("kpsewhich").arg(name).output().unwrap();
    assert!(output.status.success(), "missing fixture font {name}");
    std::fs::read(String::from_utf8(output.stdout).unwrap().trim()).unwrap()
}

#[tokio::test]
#[ignore = "requires Linux isolation, native TeX packages and fonts"]
async fn custom_resume_class_package_tikz_tables_and_cover_letter_compile() {
    let mut files = text(include_str!("../fixtures/resume/main.tex"));
    files.insert(
        "resume.cls".into(),
        include_bytes!("../fixtures/resume/resume.cls").to_vec(),
    );
    files.insert(
        "details.sty".into(),
        include_bytes!("../fixtures/resume/details.sty").to_vec(),
    );
    let output = compile(files, Engine::PdfLatex).await;
    assert_eq!(output.page_count, 1);
    let pdf = lopdf::Document::load_mem(&output.pdf).unwrap();
    assert!(pdf.extract_text(&[1]).unwrap().contains("Alex Example"));
    let output = compile(
        text(include_str!("../fixtures/cover-letter/main.tex")),
        Engine::PdfLatex,
    )
    .await;
    assert_eq!(output.page_count, 1);
}

#[tokio::test]
#[ignore = "requires Linux isolation and Unicode fonts"]
async fn uploaded_opentype_and_truetype_fonts_keep_relative_paths() {
    for (font_name, extension) in [("lmroman10-regular.otf", "otf"), ("Vera.ttf", "ttf")] {
        let bytes = font(font_name);
        for engine in [Engine::XeLatex, Engine::LuaLatex] {
            let source = format!(
                r"\documentclass{{article}}\usepackage{{fontspec}}\setmainfont[Path=fonts/]{{Regular.{extension}}}\begin{{document}}Uploaded font café\end{{document}}"
            );
            let mut files = text(&source);
            files.insert(format!("fonts/Regular.{extension}"), bytes.clone());
            let output = compile(files, engine).await;
            assert_eq!(output.page_count, 1);
            let pdf = lopdf::Document::load_mem(&output.pdf).unwrap();
            assert!(pdf.extract_text(&[1]).unwrap().contains("Uploaded font"));
        }
    }
}

#[tokio::test]
#[ignore = "requires Linux sandbox with latexmk, BibTeX and Biber"]
async fn bibliography_backends_resolve_citations() {
    assert!(std::path::Path::new("/usr/bin/latexmk").exists());
    assert!(std::path::Path::new("/usr/bin/biber").exists());
    for source in [
        include_str!("../fixtures/bibliography/bibtex.tex"),
        include_str!("../fixtures/bibliography/biber.tex"),
    ] {
        let mut files = text(source);
        files.insert(
            "references.bib".into(),
            include_bytes!("../fixtures/bibliography/references.bib").to_vec(),
        );
        let output = compile(files, Engine::PdfLatex).await;
        let pdf = lopdf::Document::load_mem(&output.pdf).unwrap();
        let text = pdf
            .extract_text(&(1..=output.page_count).collect::<Vec<_>>())
            .unwrap();
        assert!(text.contains("Example Research"), "{text}");
        assert!(
            !output.warnings.iter().any(|w| w.contains("undefined")),
            "{:?}",
            output.warnings
        );
    }
}

#[tokio::test]
#[ignore = "requires Linux sandbox and TeX"]
async fn missing_fonts_assets_and_invalid_source_return_errors() {
    for (source, engine) in [
        (
            r"\documentclass{article}\usepackage{graphicx}\begin{document}\includegraphics{missing-image}\end{document}",
            Engine::PdfLatex,
        ),
        (
            r"\documentclass{article}\usepackage{fontspec}\setmainfont{TexCompilerMissingFont123}\begin{document}Text\end{document}",
            Engine::XeLatex,
        ),
        (
            r"\documentclass{article}\usepackage{texcompilerMissingPackage}\begin{document}Text\end{document}",
            Engine::PdfLatex,
        ),
    ] {
        assert!(matches!(
            compiler()
                .compile(
                    Project::new(text(source), "main.tex".into()).unwrap(),
                    engine
                )
                .await,
            Err(NativeError::Compilation(_))
        ));
    }
}
