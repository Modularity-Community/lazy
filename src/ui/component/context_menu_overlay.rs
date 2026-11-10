use iced::{
    Border, Color, Event, Point, Size,
    advanced::{Clipboard, Layout, Overlay, Shell, layout, mouse::Cursor, renderer},
    mouse,
};

const MENU_WIDTH: f32 = 180.0;
const MENU_HEIGHT: f32 = 110.0;
const MARGIN: f32 = 8.0;

pub struct ContextMenuOverlay<Message> {
    pub position: Point,
    pub on_close: Message,
}

impl<Message: Clone> Overlay<Message, iced::Theme, iced::Renderer> for ContextMenuOverlay<Message> {
    fn layout(&mut self, _renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let width = MENU_WIDTH.min(bounds.width);
        let height = MENU_HEIGHT.min(bounds.height);

        let max_x = (bounds.width - width - MARGIN).max(0.0);
        let max_y = (bounds.height - height - MARGIN).max(0.0);

        let x = self.position.x.clamp(0.0, max_x);
        let y = self.position.y.clamp(0.0, max_y);

        layout::Node::new(Size::new(width, height)).move_to(Point::new(x, y))
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
    ) {
        use iced::advanced::Renderer as _;

        let bounds = layout.bounds();

        renderer.with_layer(bounds, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        color: Color::from_rgb8(97, 87, 125),
                        width: 1.0,
                        radius: 1.0.into(),
                    },
                    ..Default::default()
                },
                Color::from_rgb8(38, 39, 54),
            );
        });
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let clicked = matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left | mouse::Button::Right
            ))
        );

        if !clicked {
            return;
        }

        let inside = cursor
            .position()
            .is_some_and(|p| layout.bounds().contains(p));

        if !inside {
            shell.publish(self.on_close.clone());
        }

        shell.capture_event();
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor
            .position()
            .is_some_and(|p| layout.bounds().contains(p))
        {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
}
