fn main() {
    // Intentionally empty. DB is kept across builds; wipe only with --fresh at runtime.
    println!("cargo:rerun-if-changed=build.rs");
}
