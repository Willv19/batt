#[derive(Debug)]
pub struct Config {
    pub batteries: Vec<String>,
    pub delay_seconds: u64,
    pub warning: u8,
    pub critical: u8,
    pub danger: u8,
    pub dangercmd: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            batteries: vec![String::from("BAT0"), String::from("BAT1")],
            delay_seconds: 60,
            warning: 25,
            critical: 10,
            danger: 3,
            dangercmd: String::from("notify-send -u critical 'Critical' 'Hibernated system due to low battery' && systemctl hibernate"),
        }
    }
}
