use anstyle::{AnsiColor, Color, Style};
use clap::{builder::Styles, Parser};

/// Generate a video clip from the command-line in a configurable way.
///
/// https://github.com/siph/clipwhisper
#[derive(Debug, Parser, Clone)]
#[clap(name = "clipwhisper", version)]
#[command(styles=get_styles())]
pub struct Args {
    /// The path to the video that you intend to clip. This file will WILL be modified if both
    /// `--input` and `--output` are the same value.
    #[arg(short, long)]
    pub input: String,

    /// The path for the final video output. THIS WILL OVERWRITE EXISTING FILES!
    #[arg(short, long)]
    pub output: String,

    /// The length in seconds of the final desired clip. A duration that exceeds the remaining
    /// video runtime will be bound within the available duration, resulting in a clip that is
    /// shorter than the provided duration.
    #[arg(short, long, default_value_t = 10)]
    pub duration: u32,

    /// Denote in seconds where the clip should begin. An offset that surpasses the length of the
    /// input video will be bound to the available duration and result in an empty or very short
    /// clip.
    #[arg(short = 's', long, default_value_t = 0)]
    pub offset: u32,
}

fn get_styles() -> Styles {
    Styles::styled()
        .usage(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Magenta))),
        )
        .header(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Magenta))),
        )
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Blue))))
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::White))))
}
