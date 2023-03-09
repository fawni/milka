#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!(
            "{}{}{} {}",
            "[".bright_black(),
            "+".green(),
            "]".bright_black(),
            format!($($arg)*))
    };
}

#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {
        println!(
            "{}{}{} {}",
            "[".bright_black(),
            "-".red(),
            "]".bright_black(),
            format!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        println!(
            "{}{}{} {}",
            "[".bright_black(),
            "!".yellow(),
            "]".bright_black(),
            format!($($arg)*))
    };
}
