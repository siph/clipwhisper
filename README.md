# clipwhisper

`clipwhisper` is designed to simplify the process of creating video clips from
a source video file. Whether you want to extract specific scenes, segments, or
highlights from a longer video or prepare content for social media platforms,
this tool provides a straightforward and efficient solution.


## Dependencies

- ffmpeg
- ffprobe


## Installation

### Cargo

Clipwhisper can be built through `Cargo` with:

```bash
cargo build --release
```


### Nix

This repository includes a flake and derivation that can be ran with:

```bash
nix run github:siph/clipwhisper#default -- --help
```


## Usage

Help and usage information can be displayed by running `clipwhisper` with the help flag:

```bash
clipwhisper --help
```


## Example

Produce a `10` second clip starting at the `30` second mark:

```bash
clipwhisper --input source_video.mp4 --offset 30 --duration 10 --output clip.mp4
```

