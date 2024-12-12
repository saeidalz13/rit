pub const BOLD_BLACK: &str = "\x1b[1;30m"; // Bold Black
pub const BOLD_RED: &str = "\x1b[1;31m"; // Bold Red
pub const BOLD_GREEN: &str = "\x1b[1;32m"; // Bold Green
pub const BOLD_YELLOW: &str = "\x1b[1;33m"; // Bold Yellow
pub const BOLD_BLUE: &str = "\x1b[1;34m"; // Bold Blue
pub const BOLD_PURPLE: &str = "\x1b[1;35m"; // Bold Purple
pub const BOLD_CYAN: &str = "\x1b[1;36m"; // Bold Cyan
pub const BOLD_WHITE: &str = "\x1b[1;37m"; // Bold White
pub const FORMAT_RESET: &str = "\x1b[0m"; // Reset formatting

pub fn print_success_msg(msg: &'static str) {
    println!("{}[SUCCESS]{} {}", BOLD_GREEN, FORMAT_RESET, msg)
}
