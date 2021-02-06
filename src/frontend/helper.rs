#[inline(always)]
pub fn log_error(prefix: &str, msg: String) {
    use ansi_term::Color::Red;

    println!("[{}]: {}", Red.paint(prefix), msg);
}

#[inline(always)]
pub fn log_success(prefix: &str, msg: String) {
    use ansi_term::Color::Green;

    println!("[{}]: {}", Green.paint(prefix), msg);
}

#[inline(always)]
pub fn log_info(prefix: &str, msg: String) {
    use ansi_term::Color::Blue;

    println!("[{}]: {}", Blue.paint(prefix), msg);
}
