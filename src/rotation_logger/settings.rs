//! # Settings and support data for `Logger` setup.
//!
//! `Logs Formatter` support five `Mask Types`(mask_type) you can operate with:
//! - timestamp: represent timestamp of logged data. Time will be taken when logged message received by logger, so it not 100% accurate when event occurred.
//! - splitter: represent splitter symbol which will separate every `Mask`
//! - modules: list of modules that was source of log data
//! - message: log message it self
//!
//! Each `Mask Type` except `splitter` accept format syntax after `:` char:
//! `{<mask_type:<mask_length>_<mask_width>_<mask_align>>}`
//! - mask_length: length of string. On positive value limit string length from begin, on negative value from end.
//! - mask_width: width of column for this Mask Type.
//! - mask_align: vertical align for text on this column. Possible values: left, center, right.
//!
//! # Example:
//!
//! ```
//! MessageFormatter::new(
//!     "::",
//!     "{timestamp:-6:30:right}{splitter}{modules:_:_:left}{splitter}{message}",
//!     "%Y-%m-%d %H:%M:%S.%f",
//! );
//!
//! ```
//!
//! `Logs Output` supported options: file, console, auto
//! - file: all logs data will be store to logs file with declared settings.
//! - console: output to console
//! - auto: will use console when in develop mode and file on release.
//!
//! # Example:
//!
//! ```
//! OutputChannel::file(
//!     "./".into(),
//!     10,
//!     FileSize::from_megabytes(5),
//!     "new_logger".into(),
//!     "log".into(),
//! );
//! ```
//!
use std::{cmp::min, path::PathBuf};

use chrono::Local;

use crate::rotation_logger::logger::Message;

/// Settings for data format and output of `Logger`.
/// All Settings must be set before `Logger` start and cant be changed during work.
/// `Enabled` or `Disabled` `Logger` can be used to log data, but in case of `Disabled Logger` nothing will happen.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Setting initial Logger Type
    is_enabled: bool,
    /// Format for output logging string
    formatter: MessageFormatter,
    /// Output direction to store logs
    output: OutputChannel,
    /// Accumulating buffer size.
    /// Buffer actually is a Vec<String>::len window, which will be accumulated before flushing into file.
    buffer_size: usize,
}

impl Settings {
    pub fn new(
        is_enabled: bool,
        buffer_size: usize,
        output: OutputChannel,
        formatter: MessageFormatter,
    ) -> Self {
        Self {
            is_enabled,
            output,
            formatter,
            buffer_size,
        }
    }

    pub fn format_message(&self, message: &Message) -> String {
        self.formatter.format(message)
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn output(&self) -> &OutputChannel {
        &self.output
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            is_enabled: true,
            output: Default::default(),
            formatter: Default::default(),
            buffer_size: 2048,
        }
    }
}

/// File Size wrapper for easier declaration
/// Store bits size.
/// Inner data stored as Bits value.
#[derive(Debug, Clone)]
pub struct FileSize {
    size: usize,
}

impl FileSize {
    pub fn from_bytes(bytes: usize) -> Self {
        Self { size: bytes * 8 }
    }

    pub fn from_kilobytes(kilobytes: usize) -> Self {
        Self {
            size: kilobytes * 8 * 1000,
        }
    }
    pub fn from_megabytes(megabytes: usize) -> Self {
        Self {
            size: megabytes * 8 * 1000 * 1000,
        }
    }
    pub fn from_gigabytes(gigabytes: usize) -> Self {
        Self {
            size: gigabytes * 8 * 1000 * 1000 * 1000,
        }
    }
}

impl Default for FileSize {
    fn default() -> Self {
        Self::from_megabytes(2)
    }
}

impl PartialEq<u64> for FileSize {
    fn eq(&self, other: &u64) -> bool {
        *other == self.size as u64
    }
}

/// Formatted for Log Message.
#[derive(Debug, Clone)]
pub struct MessageFormatter {
    /// Timestamp format.
    /// Support Chrono timestamp formats.
    timestamp: String,
    /// List of parsed Mask with set format values.
    _masks: Vec<FormatMask>,
    /// SPlitter symbols
    splitter: String,
}

impl Default for MessageFormatter {
    fn default() -> Self {
        let format = "{timestamp} {splitter} {modules} {splitter} {message}";
        Self {
            timestamp: "%Y-%m-%d %H:%M:%S.%f".to_string(),
            splitter: "::".into(),
            _masks: Self::_set_masks(format),
        }
    }
}

impl MessageFormatter {
    pub fn new(splitter: &str, format: &str, timestamp: &str) -> Self {
        Self {
            timestamp: timestamp.into(),
            splitter: splitter.into(),
            _masks: Self::_set_masks(format),
        }
    }

    /// Process input message with rules.
    pub fn format(&self, message: &Message) -> String {
        let mut result = "".to_string();

        let timestamp = if !self.timestamp.is_empty() {
            let timestamp = Local::now().format(&self.timestamp).to_string();
            // let timestamp = timestamp[0..timestamp.len() - self.timestamp.limiter()].to_string();
            timestamp
        } else {
            "".to_string()
        };

        for mask in &self._masks {
            match &mask.mask_type {
                MaskType::Raw(value) => result = format!("{result}{value}"),
                MaskType::Timestamp => {
                    let timestamp = self._format_by_length(&timestamp, &mask.length);
                    let timestamp =
                        self._format_by_width_align(&timestamp, &mask.width, &mask.align);
                    result = format!("{result}{timestamp}");
                }
                MaskType::Message => {
                    let message = self._format_by_length(&message.text(), &mask.length);
                    let message = self._format_by_width_align(&message, &mask.width, &mask.align);
                    result = format!("{result}{message}");
                }
                MaskType::Splitter => {
                    result = format!("{result}{}", self.splitter);
                }
                MaskType::Modules => {
                    let modules = message
                        .modules()
                        .join(format!("{}", self.splitter).as_str());

                    let modules = self._format_by_length(&modules, &mask.length);
                    let modules = self._format_by_width_align(&modules, &mask.width, &mask.align);
                    result = format!("{result}{modules}");
                }
            }
        }
        result
    }

    fn _format_by_length(&self, value: &str, length: &i32) -> String {
        if *length > 0 {
            value[0..min(*length as usize, value.len())].to_string()
        } else {
            value[0..value.len() - min(length.unsigned_abs() as usize, value.len())].to_string()
        }
    }

    fn _format_by_width_align(&self, value: &str, width: &usize, align: &TextAlign) -> String {
        if value.len() >= *width {
            return value[0..*width].to_string();
        };

        let free_space = width - value.len();
        let (left_space, right_space) = match align {
            TextAlign::Left => ("".to_string(), " ".repeat(free_space)),
            TextAlign::Center => {
                let half = (free_space / 2) as usize;
                (" ".repeat(half), " ".repeat(free_space - half))
            }
            TextAlign::Right => (" ".repeat(free_space), "".to_string()),
        };
        format!("{left_space}{value}{right_space}")
    }

    fn _set_masks(format: &str) -> Vec<FormatMask> {
        let mut result = vec![];
        let format = format.to_string();
        let mut format = format.as_str();
        if !format.contains("{") || !format.contains("}") {
            panic!("Format String wrong syntax: {format}")
        }
        while !format.is_empty() {
            let opening_delimiter = format.find("{");
            if let None = opening_delimiter {
                result.push(FormatMask::from(format));
                return result;
            }
            let opening_delimiter = opening_delimiter.unwrap();
            if format[0..opening_delimiter].to_string() != "" {
                result.push(FormatMask::from(&format[0..opening_delimiter]));
            }

            let close_delimiter = format.find("}");
            if let None = close_delimiter {
                result.push(FormatMask::from(format));
                return result;
            }
            let close_delimiter = close_delimiter.unwrap();
            let scoped_value = &format[opening_delimiter + 1..close_delimiter];
            result.push(FormatMask::from(scoped_value));
            format = &format[close_delimiter + 1..format.len()];
        }
        result
    }
}

/// Format Mask with rules.
#[derive(Debug, Clone)]
struct FormatMask {
    mask_type: MaskType,
    length: i32,
    width: usize,
    align: TextAlign,
}

impl From<&str> for FormatMask {
    fn from(value: &str) -> Self {
        let splitted_data: Vec<&str> = value.split(":").collect();
        if splitted_data.len() > 4 {
            panic!("Wrong Mask format: {value}")
        }

        let default_width = 30;
        let default_length = 30;
        let mask_type = splitted_data[0];
        let length = splitted_data
            .get(1)
            .unwrap_or(&default_length.to_string().as_str())
            .parse::<i32>()
            .unwrap_or(default_length);
        let width = splitted_data
            .get(2)
            .unwrap_or(&default_width.to_string().as_str())
            .parse::<usize>()
            .unwrap_or(default_width as usize);
        let align = *splitted_data.get(3).unwrap_or(&"center");

        Self {
            mask_type: MaskType::from(mask_type),
            length,
            width,
            align: TextAlign::from(align),
        }
    }
}

/// Type of Format Masks
#[derive(Debug, Clone)]
enum MaskType {
    Raw(String),
    Timestamp,
    Message,
    Splitter,
    Modules,
}

impl From<&str> for MaskType {
    fn from(value: &str) -> Self {
        if value.to_lowercase() == "timestamp" {
            Self::Timestamp
        } else if value.to_lowercase() == "splitter" {
            Self::Splitter
        } else if value.to_lowercase() == "modules" {
            Self::Modules
        } else if value.to_lowercase() == "message" {
            Self::Message
        } else {
            Self::Raw(value.to_string())
        }
    }
}

/// Text horizontal align.
#[derive(Debug, Clone)]
enum TextAlign {
    Left,
    Center,
    Right,
}

impl From<&str> for TextAlign {
    fn from(value: &str) -> Self {
        if value.to_lowercase() == "left" {
            Self::Left
        } else if value.to_lowercase() == "right" {
            Self::Right
        } else if value.to_lowercase() == "center" {
            Self::Center
        } else {
            Self::Center
        }
    }
}

/// Output Types for Logger.
#[derive(Debug, Clone)]
pub enum OutputChannel {
    /// Store to files.
    File(FileSettings),
    /// Output to stdout.
    Console,
    /// If dev mode -> stdout, If release -> file
    Auto(FileSettings),
}

impl Default for OutputChannel {
    fn default() -> Self {
        Self::Console
    }
}

impl OutputChannel {
    pub fn console() -> Self {
        Self::Console
    }
    pub fn auto() -> Self {
        Self::Console
    }
    pub fn file(
        path: PathBuf,
        capacity: usize,
        file_size: FileSize,
        filename: String,
        file_extension: String,
    ) -> Self {
        Self::File(FileSettings::new(
            path,
            capacity,
            file_size,
            filename,
            file_extension,
        ))
    }

    pub fn settings(&self) -> Option<&FileSettings> {
        match &self {
            OutputChannel::File(file_output) => Some(file_output),
            OutputChannel::Console => None,
            OutputChannel::Auto(file_output) => Some(file_output),
        }
    }
}

/// Settings for logs files and rotation.
#[derive(Debug, Clone)]
pub struct FileSettings {
    path: PathBuf,
    capacity: usize,
    file_size: FileSize,
    filename: String,
    file_extension: String,
}

impl FileSettings {
    pub fn new(
        path: PathBuf,
        capacity: usize,
        file_size: FileSize,
        filename: String,
        file_extension: String,
    ) -> Self {
        Self {
            path,
            capacity,
            file_size,
            filename,
            file_extension,
        }
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn filename(&self) -> &String {
        &self.filename
    }
    pub fn file_extension(&self) -> &String {
        &self.file_extension
    }
    pub fn file_size(&self) -> u64 {
        self.file_size.size as u64
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Default for FileSettings {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./logs"),
            capacity: 10,
            file_size: Default::default(),
            filename: "logger".into(),
            file_extension: "log".into(),
        }
    }
}
