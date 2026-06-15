use crate::db::{load, read};
use dynamism::umap::FittedChunks;
use iced::Alignment;
use iced::widget::scrollable;
use iced::widget::{button, column, container, row, stack, text, text_input};
use iced::{Element, Length, Task};
use iced_plot::{
    HoverPickEvent, PlotUiMessage, PlotWidget, PlotWidgetBuilder, PointId, Series, ShapeId,
};

#[derive(Debug, Clone)]
pub enum Message {
    Welcome,
    ButtonPressed,
    ContentChanged(String),
    Plot(PlotUiMessage),
    DB(Vec<FittedChunks>),
}

pub struct Dynamism {
    pub widget: PlotWidget,
    pub series_id: Option<ShapeId>,
    pub points: Option<Vec<FittedChunks>>,
    pub picked_index: Option<usize>,
    pub content: String,
}

impl Dynamism {
    pub fn new() -> Self {
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

        Self {
            content: String::new(),
            widget,
            series_id: None,
            points: None,
            picked_index: None,
        }
        //let load = Task::perform(load("rustlang".to_string()), |result| match result {
        //    Ok(()) => Message::DB,
        //    _ => Message::DB,
        //});
        //let load = Task::perform(async {}, |_| Message::DB);
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ContentChanged(content) => self.content = content,
            Message::Welcome => todo!(),
            Message::ButtonPressed => {
                println!("{}", self.content.to_string());
                let query = self.content.clone();

                let task = Task::perform(
                    async move {
                        load(query).await.unwrap();

                        let mut dir = std::env::current_dir().unwrap();
                        dir.push("db/");

                        Ok(read(dir.to_str().unwrap().to_string()).await)
                    },
                    |result| match result {
                        Ok(records) => Message::DB(records),
                        Err(err) => err,
                    },
                );
                self.content.clear();
                return task;
            }
            Message::DB(chunks) => {
                self.points = Some(chunks);
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
        if let Some(p) = self.picked_index {
            let chunk = &self.points.as_ref().unwrap()[p];
            container(scrollable(
                column![
                    text("URL:")
                        .size(12)
                        .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                    text(&**chunk.url).size(14),
                    text("Snippet").size(12),
                    text(&**chunk.snippet).size(14),
                    text("Text:")
                        .size(12)
                        .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                    text(&**chunk.text).size(14),
                ]
                .spacing(5),
            ))
            .width(350)
            .height(Length::Fill)
            .padding(15)
            .into()
        } else {
            container("").width(0).height(Length::Fill).into()
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let mut graph_row = row![self.widget.view().map(Message::Plot)];

        if self.picked_index.is_some() {
            graph_row = graph_row.push(self.detail_panel());
        }

        let graph = graph_row.width(Length::Fill).height(Length::Fill);

        let input = container(row![
            text_input("test", &self.content)
                .on_input(Message::ContentChanged)
                .width(200),
            button("test").on_press(Message::ButtonPressed)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Start)
        .align_y(Alignment::End)
        .padding(20);
        stack![graph, input]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
