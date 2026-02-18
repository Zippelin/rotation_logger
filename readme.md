### Rotation logger.

This is simple async rotation Logger lib. It can be safely used across threads by passing logger as argument or using macros.

## Example:

- First you need to configure Logger. This will tell Logger how to format log strings and time.

    ```rust
    let formatter = MessageFormatter::new(
        "::",
        "{timestamp:-6:30:right}{splitter}{modules:_:_:left}{splitter}{message}",
        "%Y-%m-%d %H:%M:%S.%f",
    );
    ```

    Configuration contains:
    - splitter symbol - this symbol will split all separate data.
    - format log string - this represent string rules. Each rule (mask) except `splitter`, surrounded by curly bracers support formatting values split by `:` char.

        Possible `Masks`:
        - timestamp: place where time will be printed.
        - splitter: place where to use `splitter`.
        - modules: modules which was the source of log data, will be splitter by `splitter`.
        - message: log message eit self.

        Formatting values inside curly bracers split by char `:` use rule:  
         `{<mask_type>:<string_length>:<column_width>:<text_halign>}`.
        - string_length: limit string length, in case on positive value limit from start in case of negative value from end.
        - column_width: width of column. If data cant fir to column it will be sliced.
        - text_halign: horizontal text align: left, right, center.

        All this formatting rules could be skipped all together or any one of them with char `_`:  
         `{<mask_type>:_:<column_width>:_}`

    - time timestamp - timestamp from `chrono` Mask `timestamp`.

- Then you need to decide where to store logs: `file` or `console` or `auto` - leave decision on logger (console on dev mode and file on release version).

    ```rust
    let output = OutputChannel::file(
        "./".into(),
        10,
        FileSize::from_megabytes(5),
        "new_logger".into(),
        "log".into(),
    );
    ```

- Now we can create Log Setting.
  First flag with `true` mean we have enabled logger. We also can choose disabled logger and all log method will be still supported but all inner work will skipped.

    ```rust
    let settings = Settings::new(true, 5, output, formatter);
    ```

- Creating new Logger instance. We can safely `.clone()` instance to pass it in other threads.

    ```rust
    let logger = Logger::new(settings);

    let logger_logger_01 = logger.clone();
    let logger_logger_02 = logger.clone();
    ```

- Logger ready to work.
  You can join thread or wait any other long live thread.

    ```rust
    let joiner = logger.run_async();
    ```

- You can pass cloned logger to other thread.

    ```rust
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
    ```

For full example look at [Demo](./examples/demo.rs)
