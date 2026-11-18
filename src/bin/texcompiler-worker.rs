#![forbid(unsafe_code)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    texcompiler::compiler::worker_main()
}
