use std::path::PathBuf;

use itertools::Itertools;

fn main() -> miette::Result<()> {
    let deepstream_include_base = "/opt/nvidia/deepstream/deepstream/sources/includes";
    let mut includes = [deepstream_include_base]
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    let glib_pkg = pkg_config::probe_library("gstreamer-1.0").unwrap();
    includes.extend(glib_pkg.include_paths.iter().cloned());

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut bindings_builder = bindgen::Builder::default()
        .blocklist_file(".*glib.*")
        // The input header we would like to generate
        // bindings for.
        .header_contents(
            "bindings.h",
            vec![
                "gstnvdsmeta.h",
                "nvds_version.h",
                "nvdsmeta_schema.h",
                "nvdsmeta.h",
                "nvds_roi_meta.h",
                "nvds_tracker_meta.h",
                "nvbufsurface.h",
            ]
            .iter()
            .map(|nvds_header| format!(r##" #include "{deepstream_include_base}/{nvds_header}" "##))
            .join("\n")
            .as_str(),
        )
        // .clang_arg("-x c++")
        // .clang_arg("-std=c++14")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    for path in &includes {
        println!("cargo:rustc-link-search={}", path.display());
        bindings_builder = bindings_builder.clang_arg(format!("-I{}", path.display()));
    }

    let bindings = bindings_builder
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");
    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    Ok(())
}
