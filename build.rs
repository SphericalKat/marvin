fn main() {
    cargo_emit::rerun_if_changed!("migrations",);
}
