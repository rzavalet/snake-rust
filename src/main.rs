use std::process;

/// Entry point.
fn main() {
    let font_path = get_font_path();

	if let Err(e) = snake_rust::run(&font_path) {
        eprintln!("{:?}", e);
        process::exit(1);
    }
    else {
        process::exit(0);
    }
}

/// Returns the path of the font file we are going to use for rendering text in SDL.
/// FIXME: Return some kind of "Path" object instead of a String.
/// TODO:  Validate the path actually exists.
///
fn get_font_path() -> String {

    if let Some(project_root) = option_env!("CARGO_MANIFEST_DIR") {
        // FIXME: this only works in the development environment,
        // we should make the font file part of the distribution.
        let mut path = String::from(project_root);
        path.push_str("/res/Roboto-Regular.ttf");
        path
    }
    else {
        // As a fallback, use the system-wide path for the font in the Debian package `fonts-roboto`:
        String::from("/usr/share/fonts/truetype/roboto/unhinted/RobotoTTF/Roboto-Regular.ttf")
    }
}

