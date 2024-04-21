use std::process::Command;

macro_rules! exec {
    ($cmd:tt $($args:tt)*) => {
        Command::new($cmd).args([$($args),*]).status().map(|s| s.success()).unwrap_or(false)
    };
}

fn main() -> Result<(), i32> {
    if !exec!("cargo" "build" "--bin" "baby" "--target" "wasm32-unknown-unknown" "--release") {
        eprintln!("note: rust wasm32-unknown-unknown target can be installed with:");
        eprintln!("note:     rustup target add wasm32-unknown-unknown");
        return Err(1);
    };

    if !exec!("wasm-bindgen" "--target" "web" "--out-dir" "./web/out"
                "./target/wasm32-unknown-unknown/release/baby.wasm")
    {
        eprintln!("note: wasm-bindgen can be installed with:");
        eprintln!("note:     cargo install wasm-bindgen");
        return Err(1);
    };

    if !exec!("zip" "-r" "baby.zip" "web/*") {
        return Err(1);
    }

    Ok(())
}
