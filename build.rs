fn main() {
    // need this to find SDL libs on Windows
    println!("cargo::rustc-link-search=native=lib");
    // don't need this?
    // println!("cargo::rustc-link-lib=SDL2");
}
