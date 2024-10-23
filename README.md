# tiyt (Terminal Interface for Youtube)

Yeah, i also don't like the name, but it is what it is.

Just a simple [tui](https://en.wikipedia.org/wiki/Text-based_user_interface) application for accessing Youtube thought the terminal.

## Reference

- [ratatui](https://ratatui.rs)
- [crossterm](https://github.com/crossterm-rs/crossterm)

## Dependencies

- mpv (for playing the videos)

## Development

Building:

```shell
cargo build
```

Building and running:

```shell
cargo run
```

You need to create a `.env` to store your Youtube api key,
also, you can give mpv custom flag options like so:

```shell
API_KEY=<insert_your_api_key>
MPV_OPTION='--fs --ytdl-format=best[height<=720]'
```
