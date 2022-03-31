use gtk4::gio;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    tonic_build::configure()
        .compile(
            &["../proto/services.proto"],
            &["../proto/"]
        )
        .unwrap();

    gio::compile_resources(
        "res",
        "res/resources.gresource.xml",
        "soundbase.gresource"
    );

    Ok(())
}