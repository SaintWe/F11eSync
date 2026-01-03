#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.compile().expect("failed to embed Windows icon");
}

#[cfg(not(windows))]
fn main() {}

