use std::process::Command;

fn main() {
		let head_sha = String::from_utf8(Command::new("git")
				.arg("rev-parse")
				.arg("HEAD")
				.output().unwrap().stdout).unwrap().trim().to_string();

		let mut patch_sha = String::from_utf8(Command::new("bash")
				.arg("-c")
				.arg("git diff | sha1sum | awk '{print $1}'")
				.output().unwrap().stdout).unwrap().trim().to_string();
		// blank diff
		if patch_sha == "da39a3ee5e6b4b0d3255bfef95601890afd80709" {
				patch_sha = String::from("-");
		}

		println!("cargo:rustc-env=HEAD_SHA={}", head_sha);
		println!("cargo:rustc-env=PATCH_SHA={}", patch_sha);
}
