fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile(
            &["src/receipt_verifier.proto"],
            &["src/"],
        )?;
    Ok(())
}