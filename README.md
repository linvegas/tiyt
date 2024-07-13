# tiyt (Terminal Interface for Youtube)

Yeah, i also don't like the name, but it is what it is.

Just a simple [tui](https://en.wikipedia.org/wiki/Text-based_user_interface) application for accessing Youtube thought the terminal.

## Reference

- [ratatui](https://ratatui.rs)
- [crossterm](https://github.com/crossterm-rs/crossterm)

## Development

Building:

```shell
cargo build
```

Building and running:

```shell
cargo run
```

mpv flags example: (use it inside a `.env` file)

```shell
MPV_OPTION='--fs --ytdl-format=best[height<=720]'
```
