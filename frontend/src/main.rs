pub mod app;
pub mod data;
pub mod model;
pub mod test;

use iced::Theme;

use app::Dynamism;

pub fn main() -> iced::Result {
    iced::application(Dynamism::new, Dynamism::update, Dynamism::view)
        .theme(Theme::Dark)
        .run()?;

    Ok(())
}
