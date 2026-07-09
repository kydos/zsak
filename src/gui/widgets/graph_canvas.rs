use iced::mouse;
use iced::widget::canvas::{
    self, Canvas, Event, Frame, Geometry, Path, Program, Stroke, Text,
};
use iced::widget::canvas::event;
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector};

#[derive(Clone, Debug)]
pub struct RenderedNode {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
    pub pos: (f32, f32),
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    Router,
    Peer,
    Client,
}

#[derive(Clone, Debug)]
pub struct RenderedEdge {
    pub from: usize,
    pub to: usize,
    pub label: String,
}

#[derive(Default, Clone, Debug)]
pub struct GraphState {
    pub nodes: Vec<RenderedNode>,
    pub edges: Vec<RenderedEdge>,
}

#[derive(Default)]
pub struct GraphCanvasState {
    pub offset: Vector,
    pub scale: f32,
    pub drag_start: Option<(Point, Vector)>,
}

impl GraphCanvasState {
    pub fn new() -> Self {
        Self {
            offset: Vector::new(0.0, 0.0),
            scale: 1.0,
            drag_start: None,
        }
    }
}

pub struct GraphCanvas<'a> {
    pub graph: &'a GraphState,
}

impl<'a> GraphCanvas<'a> {
    pub fn view<Msg: 'a>(graph: &'a GraphState) -> Element<'a, Msg>
    where
        Msg: Clone,
    {
        Canvas::new(GraphCanvasProgram { graph })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

struct GraphCanvasProgram<'a> {
    graph: &'a GraphState,
}

impl<'a, Msg> Program<Msg> for GraphCanvasProgram<'a> {
    type State = GraphCanvasState;

    fn draw(
        &self,
        state: &GraphCanvasState,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let cx = bounds.width / 2.0 + state.offset.x;
        let cy = bounds.height / 2.0 + state.offset.y;
        let s = if state.scale == 0.0 { 1.0 } else { state.scale };

        let to_screen = |x: f32, y: f32| -> Point {
            Point::new(cx + x * s, cy + y * s)
        };

        // Draw edges
        for edge in &self.graph.edges {
            if let (Some(from_node), Some(to_node)) = (
                self.graph.nodes.get(edge.from),
                self.graph.nodes.get(edge.to),
            ) {
                let p1 = to_screen(from_node.pos.0, from_node.pos.1);
                let p2 = to_screen(to_node.pos.0, to_node.pos.1);
                let path = Path::line(p1, p2);
                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_color(Color::from_rgb(0.5, 0.5, 0.6))
                        .with_width(1.5),
                );
            }
        }

        // Draw nodes
        let node_r = 28.0 * s;
        for node in &self.graph.nodes {
            let center = to_screen(node.pos.0, node.pos.1);
            let color = match node.kind {
                NodeKind::Router => Color::from_rgb(0.2, 0.4, 0.9),
                NodeKind::Peer => Color::from_rgb(0.2, 0.7, 0.3),
                NodeKind::Client => Color::from_rgb(0.5, 0.5, 0.5),
            };
            let circle = Path::circle(center, node_r);
            frame.fill(&circle, color);
            frame.stroke(
                &circle,
                Stroke::default()
                    .with_color(Color::WHITE)
                    .with_width(1.5),
            );

            // Label
            frame.fill_text(Text {
                content: node.label.clone(),
                position: Point::new(center.x, center.y + node_r + 4.0 * s),
                color: Color::WHITE,
                size: (11.0 * s).into(),
                ..Text::default()
            });
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut GraphCanvasState,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (event::Status, Option<Msg>) {
        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let scroll = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => y,
                    mouse::ScrollDelta::Pixels { y, .. } => y / 50.0,
                };
                state.scale = (state.scale + scroll * 0.1).clamp(0.1, 5.0);
                (event::Status::Captured, None)
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    state.drag_start = Some((pos, state.offset));
                }
                (event::Status::Captured, None)
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some((start_pos, start_offset)) = state.drag_start {
                    state.offset = Vector::new(
                        start_offset.x + position.x - start_pos.x,
                        start_offset.y + position.y - start_pos.y,
                    );
                    return (event::Status::Captured, None);
                }
                (event::Status::Ignored, None)
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.drag_start = None;
                (event::Status::Captured, None)
            }
            _ => (event::Status::Ignored, None),
        }
    }
}
