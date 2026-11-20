use iced::{
    Event, Point, Size,
    advanced::{Clipboard, Layout, Overlay, Shell, layout, mouse::Cursor, renderer, widget::Tree},
    mouse,
};

use iced::Element;

const MARGIN: f32 = 8.0;

pub struct ContextMenuOverlay<'a, 'b, Message> {
    pub content: &'a mut Element<'b, Message>,
    pub tree: &'a mut Tree,
    pub position: Point,
    pub on_close: Message,
}

impl<Message: Clone> Overlay<Message, iced::Theme, iced::Renderer>
    for ContextMenuOverlay<'_, '_, Message>
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);

        let child = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);

        let size = child.size();

        let max_x = (bounds.width - size.width - MARGIN).max(0.0);
        let max_y = (bounds.height - size.height - MARGIN).max(0.0);

        let x = self.position.x.clamp(0.0, max_x);
        let y = self.position.y.clamp(0.0, max_y);

        layout::Node::with_children(size, vec![child]).move_to(Point::new(x, y))
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
    ) {
        use iced::advanced::Renderer as _;

        let bounds = layout.bounds();

        renderer.with_layer(bounds, |renderer| {
            self.content.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                layout.child(0),
                cursor,
                &bounds,
            );
        });
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();

        let inside = cursor.position().is_some_and(|p| bounds.contains(p));

        let mouse_pressed = matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_)));

        if mouse_pressed && !inside {
            shell.publish(self.on_close.clone());
            shell.capture_event();
            return;
        }

        if inside || !mouse_pressed {
            self.content.as_widget_mut().update(
                self.tree,
                event,
                layout.child(0),
                cursor,
                renderer,
                clipboard,
                shell,
                &bounds,
            );
        }

        if mouse_pressed {
            shell.capture_event();
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout.child(0),
            cursor,
            &layout.bounds(),
            renderer,
        )
    }
}
