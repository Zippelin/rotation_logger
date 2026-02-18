use std::{
    fs::{self, DirEntry, File, OpenOptions},
    io::{BufWriter, Write},
    sync::mpsc::Receiver,
};

use crate::{
    FileSettings, OutputChannel,
    rotation_logger::{Settings, logger::Message},
};

/// Enabled Logger worker.
pub struct EnabledLogger {
    settings: Settings,
    receiver: Receiver<Message>,
    buffer_size: usize,
}

impl EnabledLogger {
    pub fn new(settings: Settings, receiver: Receiver<Message>) -> Self {
        Self {
            buffer_size: settings.buffer_size().clone(),
            settings,
            receiver,
        }
    }

    /// Synced runner.
    pub fn run(&self) {
        match self.settings.output() {
            OutputChannel::File(file_settings) => self.write_to_file(file_settings),
            OutputChannel::Console => self.write_to_console(),
            OutputChannel::Auto(file_settings) => {
                if cfg!(debug_assertions) {
                    self.write_to_console()
                } else {
                    self.write_to_file(file_settings)
                }
            }
        }
    }

    fn write_to_console(&self) {
        loop {
            match &self.receiver.recv() {
                Ok(message) => println!("{}", self.settings.format_message(message)),
                Err(err) => {
                    println!("Logger Channel closed. Error: {err}");
                    return;
                }
            }
        }
    }

    fn write_to_file(&self, settings: &FileSettings) {
        println!("writing to file");
        let mut buffer: Vec<String> = Vec::with_capacity(self.buffer_size);
        let mut current_file_buffer: Option<BufWriter<File>> = None;

        loop {
            match &self.receiver.recv() {
                Ok(message) => {
                    buffer.push(format!("{}", self.settings.format_message(message)));

                    if self.buffer_size > buffer.len() {
                        continue;
                    }

                    if self.check_path_or_create(settings).is_err() {
                        println!("Logger cant access to log dir.");
                        return;
                    };

                    if current_file_buffer.is_none() {
                        match self.get_create_current_log_file(settings) {
                            Ok(val) => {
                                current_file_buffer = Some(BufWriter::new(val));
                            }
                            Err(_) => {
                                println!("Logger cant access to log file.");
                                return;
                            }
                        };
                    };

                    if let Some(file_buffer) = current_file_buffer.as_mut() {
                        match file_buffer.write(format!("{}\n", buffer.join("\n")).as_bytes()) {
                            Ok(_) => {}
                            Err(err) => {
                                println!("Logger error to write to file. Error: {err}");
                                return;
                            }
                        };

                        match file_buffer.flush() {
                            Ok(_) => {}
                            Err(err) => {
                                println!("Logger error to write to file. Error: {err}");
                                return;
                            }
                        }
                        buffer.clear();

                        let _ = file_buffer.get_ref().sync_all();

                        let file_size = match file_buffer.get_ref().metadata() {
                            Ok(val) => val.len() * 8,
                            Err(_) => {
                                println!("Logger cant access to log file.");
                                return;
                            }
                        };

                        if file_size >= settings.file_size() {
                            current_file_buffer = None;

                            let mut logs = self.get_log_files(settings);

                            if logs.len() >= settings.capacity() {
                                logs = match self.delete_oldest_file(logs) {
                                    Ok(val) => val,
                                    Err(_) => {
                                        println!("Logger cant delete old logs.");
                                        return;
                                    }
                                };
                            }
                            match self.reorder_filenames(settings, logs) {
                                Ok(_) => {}
                                Err(_) => {
                                    println!("Logger cant rotate logs.");
                                    return;
                                }
                            };
                        }
                    }
                }
                Err(err) => {
                    println!("Logger Channel closed. Error: {err}");
                    return;
                }
            }
        }
    }

    fn check_path_or_create(&self, settings: &FileSettings) -> Result<(), ()> {
        match fs::exists(settings.path()) {
            Ok(is_exist) => {
                if is_exist {
                    return Ok(());
                }
                match fs::create_dir(settings.path()) {
                    Ok(_) => return Ok(()),
                    Err(_) => return Err(()),
                }
            }
            Err(_) => match fs::create_dir(settings.path()) {
                Ok(_) => return Ok(()),
                Err(_) => return Err(()),
            },
        }
    }

    fn get_create_current_log_file(&self, settings: &FileSettings) -> Result<File, ()> {
        let filename = format!("{}.{}", settings.filename(), settings.file_extension());
        let mut filepath = settings.path().clone();
        filepath.push(filename);

        match OpenOptions::new().append(true).create(true).open(filepath) {
            Ok(file) => Ok(file),
            Err(_) => Err(()),
        }
    }

    fn get_log_files(&self, settings: &FileSettings) -> Vec<DirEntry> {
        match fs::read_dir(&settings.path()) {
            Ok(dir_content) => {
                let mut filtered_files: Vec<DirEntry> = dir_content
                    .filter(|file| {
                        if let Ok(file) = file {
                            file.file_name()
                                .to_string_lossy()
                                .starts_with(settings.filename())
                                && file
                                    .file_name()
                                    .to_string_lossy()
                                    .contains(&format!(".{}", settings.file_extension()))
                        } else {
                            false
                        }
                    })
                    .map(|el| el.unwrap())
                    .collect();

                filtered_files.sort_by(|a, b| {
                    a.file_name()
                        .to_str()
                        .unwrap()
                        .cmp(b.file_name().to_str().unwrap())
                });
                return filtered_files;
            }
            Err(_) => vec![],
        }
    }

    fn delete_oldest_file(&self, mut logs: Vec<DirEntry>) -> Result<Vec<DirEntry>, ()> {
        match fs::remove_file(logs.last().unwrap().path()) {
            Ok(_) => {
                logs.remove(logs.len() - 1);
                Ok(logs)
            }
            Err(_) => Err(()),
        }
    }

    fn reorder_filenames(&self, settings: &FileSettings, logs: Vec<DirEntry>) -> Result<(), ()> {
        for i in (0..logs.len()).rev() {
            let filename = logs[i].file_name();
            let split_name: Vec<&str> = filename.to_str().unwrap().split(".").collect();
            let log_number = split_name
                .last()
                .unwrap()
                .replace(settings.file_extension(), "");

            let new_log_number = match log_number.parse::<u32>() {
                Ok(val) => val + 1,
                Err(_) => 0,
            };

            let new_filename = format!(
                "{}.{}{new_log_number}",
                split_name.first().unwrap(),
                settings.file_extension()
            );

            match fs::rename(
                format!("./{}", logs[i].path().to_str().unwrap()),
                format!("./{}/{}", settings.path().to_string_lossy(), new_filename),
            ) {
                Ok(_) => continue,
                Err(_) => return Err(()),
            }
        }

        Ok(())
    }
}
