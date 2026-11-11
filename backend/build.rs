//! Build script: tracks the migrations directory for `sqlx::migrate!`.

fn main() {
    // `sqlx::migrate!` embeds the migrations in the binary, so rebuild when they change.
    println!("cargo:rerun-if-changed=migrations");
}
