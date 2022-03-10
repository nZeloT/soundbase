fn main() -> Result<(), Box<dyn std::error::Error>>{
    tonic_build::configure()
        .compile(
            &["../proto/services.proto"],
            &["../proto/"]
        )
        .unwrap();
    Ok(())
}