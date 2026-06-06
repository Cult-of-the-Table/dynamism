use iced::{Element, Task, Theme, widget::container};

pub fn main() -> iced::Result {
    iced::application(Dynamism::new, Dynamism::update, Dynamism::view)
        .theme(Theme::Dark)
        .run()?;

    Ok(())
}
#[derive(Debug, Clone)]
enum Message {
    Welcome,
}

struct Dynamism {}

impl Dynamism {
    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        container("test").style(container::rounded_box).into()
    }
}
