use crate::data::load;
use dynamism::umap::FittedChunks;
use iced::widget::{column, row};
use iced::{Element, Length, Task};
use iced_plot::{
    HoverPickEvent, PlotUiMessage, PlotWidget, PlotWidgetBuilder, PointId, Series, ShapeId,
};

use crate::test::generate_demo_data;

#[derive(Debug, Clone)]
pub enum Message {
    Welcome,
    Plot(PlotUiMessage),
    DB,
}

pub struct Dynamism {
    pub widget: PlotWidget,
    pub series_id: Option<ShapeId>,
    pub points: Option<Vec<FittedChunks>>,
    pub picked_index: Option<usize>,
}

impl Dynamism {
    pub fn new() -> (Self, Task<Message>) {
        let widget = PlotWidgetBuilder::new()
            .with_autoscale_on_updates(true)
            .with_hover_highlight_provider(|_ctx, point| {
                point.mask_padding = None;
                None
            })
            .with_pick_highlight_provider(|_ctx, point| {
                point.resize_marker(1.6);
                point.color = iced::Color::from_rgb(1.0, 0.3, 0.3);
                point.mask_padding = None;
                None
            })
            .build()
            .unwrap();

        let app = Self {
            widget,
            series_id: None,
            points: None,
            picked_index: None,
        };
        let load = Task::perform(load("rustlang".to_string()), |result| match result {
            Ok(()) => Message::DB,
            _ => Message::DB,
        });
        // let load = Task::perform(async {}, |_| Message::DB);
        (app, load)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Welcome => todo!(),
            Message::DB => {
                self.points = Some(generate_demo_data());
                let series = Series::circles(
                    self.points
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|p| [p.embeds.x, p.embeds.y])
                        .collect(),
                    5.0,
                )
                .with_label("embeddings");
                self.series_id = Some(series.id);
                let _ = self.widget.add_series(series);
            }
            Message::Plot(msg) => {
                if let Some(event) = msg.get_hover_pick_event() {
                    match event {
                        HoverPickEvent::Pick(pid) => {
                            self.picked_index = Some(pid.point_index);
                        }
                        HoverPickEvent::ClearPick => {
                            self.picked_index = None;
                        }
                        HoverPickEvent::Hover(_hid) => {}
                        HoverPickEvent::ClearHover => {}
                    }
                }
                self.widget.update(msg);
                if let Some(i) = self.picked_index {
                    self.widget.clear_pick();
                    self.widget.add_pick_point(PointId {
                        series_id: self.series_id.unwrap(),
                        point_index: i,
                    });
                }
            }
        }

        Task::none()
    }
    pub fn detail_panel(&self) -> Element<'_, Message> {
        column![match self.picked_index {
            Some(p) => &self.points.as_ref().unwrap()[p].url,
            None => "",
        }]
        .into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        row![self.widget.view().map(Message::Plot), self.detail_panel()]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
