use iced::{
    Element, Event, Length, Point, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse::Cursor, overlay, renderer, widget::Tree,
    },
    mouse,
};

use crate::ui::component::context_menu_overlay::ContextMenuOverlay;

pub struct ContextMenu<'a, Message> {
    content: Element<'a, Message>,
    position: Option<Point>,
    on_open: Box<dyn Fn(Point) -> Message + 'a>,
    on_close: Message,
}

impl<'a, Message> ContextMenu<'a, Message> {
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        position: Option<Point>,
        on_open: impl Fn(Point) -> Message + 'a,
        on_close: Message,
    ) -> Self {
        Self {
            content: content.into(),
            position,
            on_open: Box::new(on_open),
            on_close,
        }
    }
}

impl<Message: Clone> Widget<Message, iced::Theme, iced::Renderer> for ContextMenu<'_, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content]);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        layout::Node::with_children(child.size(), vec![child])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.child(0),
            cursor,
            viewport,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
        ) {
            if let Some(position) = cursor.position() {
                if layout.bounds().contains(position) {
                    shell.publish((self.on_open)(position));
                    shell.capture_event();
                    return;
                }
            }
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.child(0),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.child(0),
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        if let Some(position) = self.position {
            return Some(overlay::Element::new(Box::new(ContextMenuOverlay {
                position,
                on_close: self.on_close.clone(),
            })));
        }

        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.child(0),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: Clone + 'a> From<ContextMenu<'a, Message>> for Element<'a, Message> {
    fn from(widget: ContextMenu<'a, Message>) -> Self {
        Element::new(widget)
    }
}
