use std::process::Command;

use anyhow::Result;
use clap::Parser;
use clipwhisper::{args::Args, ClipCommand};
use env_logger::Env;
use log::{debug, info};

fn main() -> Result<()> {
    start_logger();

    let mut command: ClipCommand = Args::parse().into();

    let max_length = get_max_length(&command.input.value);

    command.target = command.target.bind_values(max_length);

    let ffmpeg_args = command.render_arguments();

    info!("Clipping video with args: {:#?}: ", &ffmpeg_args);

    let exit_status = Command::new(command.executable)
        .args(ffmpeg_args)
        .output()
        .expect("Ffmpeg command failed")
        .status;

    match exit_status.success() {
        true => Ok(()),
        false => panic!("ffmpeg ended unsuccessfully."),
    }
}

fn start_logger() {
    let env = Env::default()
        .filter_or("CLIPWHISPER_LOG_LEVEL", "info")
        .write_style_or("CLIPWHISPER_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}

/// Use `ffprobe` to get `input` length in seconds.
fn get_max_length(input: &String) -> f32 {
    let error_message = format!("Failed to get video length for: {}", &input);

    let mut command = Command::new("ffprobe");
    command.arg("-v");
    command.arg("error");
    command.arg("-show_entries");
    command.arg("format=duration");
    command.arg("-of");
    command.arg("default=noprint_wrappers=1:nokey=1");
    command.arg(input);

    debug!(
        "Running ffprobe with commands: {:#?}",
        command.get_args().collect::<Vec<_>>()
    );

    let output = command.output().expect(&error_message);

    String::from_utf8(output.stdout)
        .expect(&error_message)
        .trim()
        .parse()
        .expect(&error_message)
}
