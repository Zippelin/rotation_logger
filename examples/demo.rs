use std::{
    thread::{self, sleep},
    time::Duration,
};

use rotation_logger::{FileSize, Logger, LogsOutput, MessageFormatter, Settings};

fn main() {
    let formatter = MessageFormatter::new(
        "::",
        "{timestamp:-6:30:right}{splitter}{modules:_:_:left}{splitter}{message}",
        "%Y-%m-%d %H:%M:%S.%f",
    );

    let output = LogsOutput::file(
        "./logs".into(),
        10,
        FileSize::from_kilobytes(1),
        "new_logger".into(),
        "log".into(),
    );

    let settings = Settings::new(true, 5, output, formatter);

    let logger = Logger::new(settings);
    let joiner = logger.run_async();

    let logger_logger_01 = logger.clone();
    let logger_logger_02 = logger.clone();
    let _ = thread::spawn(move || {
        logger_logger_01.log(&vec!["THREAD1".into(), "MAIN".into()], "Starting...");

        let mut counter = 0;
        loop {
            logger_logger_01.log(
                &vec!["THREAD1".into(), "WORKER".into()],
                format!("Processing Job: {counter}").as_str(),
            );
            counter += 1;
            sleep(Duration::from_secs(1));
        }
    });

    let _ = thread::spawn(move || {
        logger_logger_02.log(&vec!["THREAD2".into(), "MAIN".into()], "Starting...");

        let mut counter = 0;
        loop {
            logger_logger_02.log(
                &vec!["THREAD2".into(), "WORKER".into()],
                format!("Processing Job: {counter}").as_str(),
            );
            counter += 2;
            sleep(Duration::from_millis(400));
        }
    });

    match joiner {
        Some(j) => {
            let _ = j.join();
        }
        None => {}
    };
}
