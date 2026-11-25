//! Exports the D3D12 Agility SDK symbols from the executable (see `D3D12SDKVersion` in `main.rs`): an
//! executable exports nothing by default, and the D3D12 loader looks them up in the process's export table.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-arg-bins=/EXPORT:D3D12SDKVersion,DATA");
        println!("cargo:rustc-link-arg-bins=/EXPORT:D3D12SDKPath,DATA");
    }
}
