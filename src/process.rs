use std::io::Write;
use std::env;

fn main() {
    dotenv::dotenv().ok();

    let mpv_option = env::var("MPV_OPTION");

    let mpv_option = match mpv_option {
        Ok(s) => s,
        Err(_) => String::new(),
    };

    let output = std::process::Command::new("mpv")
        .args(mpv_option.split_whitespace())
        .arg("https://youtube.com/watch?v=glQZlH8TfHM")
        .output()
        .expect("Failed to execute");

    std::io::stderr().write_all(&output.stderr).unwrap();
}
