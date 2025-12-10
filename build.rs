fn main() {
    println!("cargo:rustc-link-search=native=C:\\Users\\Simen\\Documents\\programmering\\luajit\\src");
    println!("cargo:rustc-link-lib=dylib=luajit-5.1");
}