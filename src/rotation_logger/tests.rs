use crate::{Message, MessageFormatter};

#[test]
fn test_message_formatter_output() {
    let variants =vec![
        ("{timestamp:-6:30:right}{splitter}{modules:_:_:left}{splitter}{message}", "       2026-02-18 15:44:00.129::Some1::Some2                  ::          test text           ".to_string(), 30),
        ("{modules:_:_:left}{splitter}{message}", "Some1::Some2                  ::          test text           ".to_string(), 0),
        ("{modules:_:_:left}{splitter}{message}{message}", "Some1::Some2                  ::          test text                     test text           ".to_string(), 0),
    ];

    let modules = vec!["Some1".into(), "Some2".into()];
    let message = Message::new(&modules, "test text");

    for (format, result, cut) in variants {
        let formatter = MessageFormatter::new("::", format, "%Y-%m-%d %H:%M:%S.%f");

        let formatted_message = formatter.format(&message);
        assert_eq!(formatted_message[cut..], result[cut..]);
    }
}
