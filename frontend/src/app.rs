use iced::widget::{column, row};
use iced::{Element, Length, Task};
use iced_plot::{
    HoverPickEvent, PlotUiMessage, PlotWidget, PlotWidgetBuilder, PointId, Series, ShapeId,
};

use crate::{model::Point, test::generate_demo_data};

#[derive(Debug, Clone)]
pub enum Message {
    Welcome,
    Plot(PlotUiMessage),
}

pub struct Dynamism {
    pub widget: PlotWidget,
    pub series_id: ShapeId,
    pub points: Vec<Point>,
    pub picked_index: Option<usize>,
}

impl Dynamism {
    pub fn new() -> Self {
        let points = generate_demo_data();
        let series = Series::circles(points.iter().map(|p| [p.x, p.y]).collect(), 5.0)
            .with_label("embeddings");
        let series_id = series.id;
        let widget = PlotWidgetBuilder::new()
            .add_series(series)
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
        Self {
            widget,
            series_id,
            points,
            picked_index: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Welcome => todo!(),
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
                        series_id: self.series_id,
                        point_index: i,
                    });
                }
            }
        }

        Task::none()
    }
    pub fn detail_panel(&self) -> Element<'_, Message> {
        column![match self.picked_index {
            Some(p) => &self.points[p].title,
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
