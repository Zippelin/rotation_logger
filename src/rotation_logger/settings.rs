use std::{cmp::min, path::PathBuf};

use chrono::Local;

use crate::rotation_logger::logger::Message;

#[derive(Debug, Clone)]
pub struct Settings {
    is_enabled: bool,
    path: PathBuf,
    capacity: usize,
    file_size: FileSize,
    filename: String,
    file_extension: String,
    formatter: MessageFormatter,
}

impl Settings {
    pub fn new(
        is_enabled: bool,
        path: PathBuf,
        capacity: usize,
        file_size: FileSize,
        filename: String,
        file_extension: String,
        formatter: MessageFormatter,
    ) -> Self {
        Self {
            is_enabled,
            path,
            capacity,
            file_size,
            filename,
            file_extension,
            formatter,
        }
    }

    pub fn format_message(&self, message: &Message) -> String {
        self.formatter.format(message)
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            is_enabled: true,
            path: PathBuf::from("./"),
            capacity: 10,
            file_size: Default::default(),
            filename: "logger".into(),
            file_extension: "log".into(),
            formatter: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileSize {
    size: usize,
}

impl FileSize {
    pub fn from_bytes(bytes: usize) -> Self {
        Self { size: bytes * 8 }
    }

    pub fn from_kilobytes(bytes: usize) -> Self {
        Self {
            size: bytes * 8 * 1000,
        }
    }
    pub fn from_megabytes(bytes: usize) -> Self {
        Self {
            size: bytes * 8 * 1000 * 1000,
        }
    }
    pub fn from_gigabytes(bytes: usize) -> Self {
        Self {
            size: bytes * 8 * 1000 * 1000 * 1000,
        }
    }
}

impl Default for FileSize {
    fn default() -> Self {
        Self::from_megabytes(2)
    }
}

#[derive(Debug, Clone)]
pub struct MessageFormatter {
    timestamp: String,
    format: String,
    _masks: Vec<FormatMask>,
    splitter: String,
}

impl Default for MessageFormatter {
    fn default() -> Self {
        let format = "{timestamp} {splitter} {modules} {splitter} {message}";
        Self {
            timestamp: "%Y-%m-%d %H:%M:%S.%f".to_string(),
            format: format.to_string(),
            splitter: "::".into(),
            _masks: Self::_set_masks(format),
        }
    }
}

impl MessageFormatter {
    pub fn new(splitter: &str, format: &str, timestamp: &str) -> Self {
        Self {
            timestamp: timestamp.into(),
            format: format.into(),
            splitter: splitter.into(),
            _masks: Self::_set_masks(format),
        }
    }

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
            TextAlign::Left => (" ".repeat(1), " ".repeat(free_space - 1)),
            TextAlign::Center => {
                let half = (free_space / 2) as usize;
                (" ".repeat(half), " ".repeat(free_space - half))
            }
            TextAlign::Right => (" ".repeat(free_space - 1), " ".repeat(1)),
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
