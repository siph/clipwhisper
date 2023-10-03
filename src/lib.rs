use std::collections::HashMap;

use args::Args;

use interpolator::{format, Formattable};
use log::{debug, warn};

pub mod args;

/// Represents a cli command to extract a clip from a video.
#[derive(PartialEq, Clone, Debug)]
pub struct ClipCommand {
    /// Command name
    pub executable: String,
    /// Input file
    pub input: CommandChunk,
    /// Filter to be ran on video track
    pub video_filter: CommandChunk,
    /// Filter to be ran on audio track
    pub audio_filter: CommandChunk,
    /// Output file
    pub output: CommandChunk,
    /// Start and end timestamps
    pub target: TargetTimeStamp,
}

impl From<Args> for ClipCommand {
    fn from(args: Args) -> Self {
        Self {
            executable: "ffmpeg".to_string(),
            input: CommandChunk {
                flag: "-i".to_string(),
                value: args.input,
            },
            video_filter: CommandChunk {
                flag: "-vf".to_string(),
                value: "select='between(t,{start},{end})',setpts=N/FRAME_RATE/TB".to_string(),
            },
            audio_filter: CommandChunk {
                flag: "-af".to_string(),
                value: "aselect='between(t,{start},{end})',asetpts=N/SR/TB".to_string(),
            },
            output: CommandChunk {
                flag: "-o".to_string(),
                value: args.output,
            },
            target: TargetTimeStamp::new(args.offset, args.duration),
        }
    }
}

impl ClipCommand {
    /// Format and display the arguments as a list of strings.
    pub fn render_arguments(&self) -> Vec<String> {
        debug!("Rendering arguments for: {:#?}", self);
        let mut arguments: Vec<String> = vec![
            self.input.clone(),
            self.video_filter.format_chunk(&self.target),
            self.audio_filter.format_chunk(&self.target),
        ]
        .into_iter()
        .flat_map(|it| vec![it.flag, it.value])
        .collect();

        // This adds a flag to overwrite any file with the same path as `output`.
        arguments.push("-y".to_string());
        // output doesn't actually have a flag in `ffmpeg`, so `-o` will break the command.
        // This just appends the value to the end of the list.
        arguments.push(self.output.value.clone());
        arguments
    }
}

/// Specifies where the clip exists within the video.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct TargetTimeStamp {
    /// Start time in seconds
    pub start: u32,
    /// End time in seconds
    pub end: u32,
}

impl TargetTimeStamp {
    /// Overflow safe `TargetTimeStamp` builder.
    pub fn new(offset: u32, duration: u32) -> Self {
        let start = offset;
        let end = match offset.overflowing_add(duration) {
            (_, true) => {
                warn!("Locking end to prevent overflow.");
                warn!("Start: {:?}", start);
                warn!("Duration: {:?}", duration);
                u32::max_value()
            }
            (end, false) => end,
        };
        Self { start, end }
    }

    /// Bind `start` and `end` values to be valid within the available `max_length`.
    pub fn bind_values(&mut self, max_length: f32) -> Self {
        debug!("Checking if values need binding: {:#?}", &self);
        // truncate decimals. I think the implication of this is that it will be impossible to get
        // the last fraction of a second in a clip. But it sure makes the math easier.
        let video_length = max_length as u32;

        // Bind `start` only if it exceeds video length.
        self.start = match video_length {
            length if self.start > length => {
                warn!(
                    "Value start `{}` exceeds video_length `{}`",
                    self.start, video_length
                );
                warn!(
                    "Binding start `{}` to video_length `{}`",
                    self.start, video_length
                );
                length
            }
            _ => self.start,
        };

        // Bind `end` only if it exceeds video length.
        self.end = match video_length {
            length if self.end > length => {
                warn!(
                    "Value end `{}` exceeds video_length `{}`",
                    self.end, video_length
                );
                warn!(
                    "Binding end `{}` to video_length `{}`",
                    self.end, video_length
                );
                length
            }
            _ => self.end,
        };
        *self
    }
}

/// Represents a key/value command segment.
#[derive(PartialEq, Clone, Debug)]
pub struct CommandChunk {
    /// Key for command
    pub flag: String,
    /// Value for command
    pub value: String,
}

impl CommandChunk {
    /// Returns `CommandChunk` with interpolated `start` and `end` for given `TargetTimeStamp`.
    pub fn format_chunk(&self, target: &TargetTimeStamp) -> Self {
        debug!("Formatting chunk: {:#?}", self);
        let formats = &[
            ("start", Formattable::display(&target.start)),
            ("end", Formattable::display(&target.end)),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        Self {
            flag: self.flag.clone(),
            value: format(&self.value, formats).expect("Failed to dynamically format argument"),
        }
    }
}

#[cfg(test)]
pub mod tests {

    use std::path::PathBuf;

    use quickcheck::Arbitrary;

    use super::*;

    #[quickcheck_macros::quickcheck]
    fn test_cli_args_into_clip_command(args: Args) {
        let result: ClipCommand = args.clone().into();
        assert_eq!(result.executable, "ffmpeg".to_string());
        assert_eq!(result.input.value, args.input);
        assert_eq!(result.output.value, args.output);
        assert_eq!(result.target.start, args.offset);
        let expected_end = match args.offset.overflowing_add(args.duration) {
            (end, false) => end,
            (_, true) => u32::max_value(),
        };
        assert_eq!(result.target.end, expected_end);
    }

    #[quickcheck_macros::quickcheck]
    fn test_target_end_is_after_start(offset: u32, duration: u32) {
        let target = TargetTimeStamp::new(offset, duration);
        assert!(target.start <= target.end);
    }

    #[quickcheck_macros::quickcheck]
    fn test_out_of_range_values_are_bound(offset: u32, duration: u32, video_length: f32) {
        let target = TargetTimeStamp::new(offset, duration).bind_values(video_length);

        // If the offset exceeds the `video_length` then it should be bound to the nearest valid
        // value, which would be the last frame of the video represented by `video_length`.
        // Otherwise it should just be offset.
        if offset > video_length as u32 {
            assert!(target.start == video_length as u32);
        } else {
            assert!(target.start == offset);
            assert!(target.start <= video_length as u32);
        }

        // `start` doesn't get moved to after `end`.
        assert!(target.start <= target.end);
        // `end` is bound within the `video_length`.
        assert!(target.end <= video_length as u32);
    }

    #[quickcheck_macros::quickcheck]
    fn test_arguments_are_formatted(command: ClipCommand) {
        let target = command.target;
        let start = target.start;
        let end = target.end;

        let video_expected = format!(
            "select='between(t,{},{})',setpts=N/FRAME_RATE/TB",
            start, end
        );
        assert_eq!(
            video_expected,
            command.video_filter.format_chunk(&target).value
        );

        let audio_expected = format!("aselect='between(t,{},{})',asetpts=N/SR/TB", start, end);
        assert_eq!(
            audio_expected,
            command.audio_filter.format_chunk(&target).value
        );
    }

    #[quickcheck_macros::quickcheck]
    fn test_argument_list_is_rendered(command: ClipCommand) {
        let expected = vec![
            command.input.flag.clone(),
            command.input.value.clone(),
            command.video_filter.flag.clone(),
            command.video_filter.format_chunk(&command.target).value,
            command.audio_filter.flag.clone(),
            command.audio_filter.format_chunk(&command.target).value,
            "-y".to_string(),
            command.output.value.clone(),
        ];
        assert!(expected.eq(&command.render_arguments()));
    }

    impl Arbitrary for ClipCommand {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Args::arbitrary(g).into()
        }
    }

    impl Arbitrary for Args {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let input = PathBuf::arbitrary(g).to_str().unwrap().to_string();
            let output = PathBuf::arbitrary(g).to_str().unwrap().to_string();
            let offset = u32::arbitrary(g);
            let duration = u32::arbitrary(g);
            Args {
                input,
                output,
                duration,
                offset,
            }
        }
    }
}
