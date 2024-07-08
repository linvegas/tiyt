fn main() {
    let _ = std::process::Command::new("mpv").args(["--fs",  "https://youtube.com/watch?v=glQZlH8TfHM"]).output();
}
