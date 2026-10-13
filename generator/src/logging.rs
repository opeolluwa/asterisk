use console::Style;

pub struct LogMessage;

#[allow(unused)]
impl LogMessage {
    pub fn error(message: &str) {
        let style = Style::new().red().bold();
        eprintln!("{}", style.apply_to(message));
    }

    pub fn success(message: &str) {
        let style = Style::new().green().bold();
        println!("{}", style.apply_to(message));
    }

    pub fn warning(message: &str) {
        let style = Style::new().yellow().bold();
        println!("{}", style.apply_to(message));
    }

    pub fn info(message: &str) {
        let style = Style::new().blue();
        println!("{}", style.apply_to(message));
    }

    pub fn neutral(message: &str) {
        let style = Style::new().white();
        println!("{}", style.apply_to(message));
    }

    pub fn step(message: &str) {
        let style = Style::new().cyan().bold();
        println!("  {} {}", style.apply_to("→"), message);
    }
}
