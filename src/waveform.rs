use crate::ARRAYLEN;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::mouse;
use iced::{Border, Color, Element, Length, Rectangle, Size};
pub struct Waveform {
    width: f32,
    color: Color,
    pos: f32,
    vals: [f32; ARRAYLEN as usize],
}

impl Waveform {
    fn new(width: f32, pos: f32, vals: [f32; ARRAYLEN as usize]) -> Self {
        Waveform {
            width,

            color: Color::BLACK,
            pos,
            vals,
        }
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

pub fn waveform(width: f32, pos: f32, vals: [f32; ARRAYLEN as usize]) -> Waveform {
    Waveform::new(width, pos, vals)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Waveform
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let max = limits.max();
        let min = limits.min();
        layout::Node::new(max)
    }

    fn draw(
        &self,
        _state: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        // Actual drawing
        const RESOLUTION: f32 = 0.01;
        const YOFFSET: f32 = 10.0;
        let bounds = layout.bounds();
        let start_x = bounds.x;
        let y = bounds.y;
        let height = bounds.height;

        let steps: i32 = ARRAYLEN;
        let step_width = bounds.width / steps as f32;

        let playing_step = ((self.pos / 100.0) * steps as f32).round() as i32;

        //Scale Averages
        let max: f32 = self
            .vals
            .iter()
            .copied()
            .max_by(|f1, f2| f1.total_cmp(f2))
            .unwrap();
        let multiplier = 1.0 / max;
        let scaled_vals: Vec<f32> = self.vals.into_iter().map(|x| x * multiplier).collect();

        for i in 0..steps {
            let val = scaled_vals[i as usize];
            let l_height = val * height;
            let color = if (playing_step - i).abs() < 4 {
                //Color::BLACK
                self.color
            } else {
                self.color
            };

            let rec = Rectangle {
                x: start_x + i as f32 * step_width,
                y: y + (height / 2.0) - (l_height / 2.0),
                width: step_width,
                height: l_height,
            };
            renderer.fill_quad(
                renderer::Quad {
                    bounds: rec,
                    border: Border {
                        color: Color::WHITE,
                        width: 0.1,
                        radius: 0.0.into(),
                    },
                    ..renderer::Quad::default()
                },
                color,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Waveform> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(waveform: Waveform) -> Self {
        Self::new(waveform)
    }
}
