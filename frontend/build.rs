use std::io::Read;
use std::path::PathBuf;

const DAISYUI_MIN_CSS_URL: &str = "https://cdn.jsdelivr.net/npm/daisyui@5";

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=static/daisyui.min.css");

	let static_dir =
		PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"))
			.join("static");

	let daisyui_min_css_filepath = static_dir.join("daisyui.min.css");

	if !daisyui_min_css_filepath.exists() {
		println!("cargo:warning=Downloading daisyui.min.css file...");

		let response = ureq::get(DAISYUI_MIN_CSS_URL)
			.call()
			.unwrap_or_else(|e| panic!("failed to download daisyui.min.css: {}", e));

		let mut file_content = Vec::new();
		response
			.into_body()
			.into_reader()
			.read_to_end(&mut file_content)
			.unwrap_or_else(|e| {
				panic!(
					"failed to receive the entire daisyui.min.css file content: {}",
					e
				)
			});

		std::fs::write(daisyui_min_css_filepath, file_content).unwrap_or_else(|e| {
			panic!("failed to save the downloaded daisyui.min.css file: {}", e)
		});
	}
}
