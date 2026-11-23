//! `sqlx::migrate!` copies the migrations into the crate when it compiles. Cargo doesn't know that
//! the crate reads them, so without this line an edited migration wouldn't trigger a rebuild.

fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
