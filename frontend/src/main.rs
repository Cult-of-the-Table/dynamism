use iced::Theme;
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    iced::daemon(u64::default(), Dynamism::update, Dynamism::view)
        .theme(Theme::Dark);
    Ok(())
}
struct Dynamism {}
enum Message {
    Welcome,
}
impl Dynamism {
    fn update(&mut self, message: Message) {}
    fn view(value: u64) {}
}
