//! Text input boundary: commits on an outside mouse press, cancels on Escape.
use iced::{Element, Event, Length, Rectangle, Size, Vector, keyboard, mouse};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse::Cursor, overlay, renderer, widget::{Operation, Tree}};

pub struct RenameField<'a, Message> {
    content: Element<'a, Message>,
    on_outside: Message,
    on_escape: Message,
}

impl<'a, Message> RenameField<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>, on_outside: Message, on_escape: Message) -> Self {
        Self { content: content.into(), on_outside, on_escape }
    }
}

impl<Message: Clone> Widget<Message, iced::Theme, iced::Renderer> for RenameField<'_, Message> {
    fn size(&self) -> Size<Length> { self.content.as_widget().size() }
    fn children(&self) -> Vec<Tree> { vec![Tree::new(&self.content)] }
    fn diff(&self, tree: &mut Tree) { tree.diff_children(&[&self.content]); }
    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let child = self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits);
        layout::Node::with_children(child.size(), vec![child])
    }
    fn operate(
        &mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0], layout.child(0), renderer, operation,
            );
        });
    }

    fn draw(&self, tree: &Tree, renderer: &mut iced::Renderer, theme: &iced::Theme, style: &renderer::Style, layout: Layout<'_>, cursor: Cursor, viewport: &Rectangle) {
        self.content.as_widget().draw(&tree.children[0], renderer, theme, style, layout.child(0), cursor, viewport);
    }
    fn update(&mut self, tree: &mut Tree, event: &Event, layout: Layout<'_>, cursor: Cursor, renderer: &iced::Renderer, clipboard: &mut dyn Clipboard, shell: &mut Shell<'_, Message>, viewport: &Rectangle) {
        // Handle outside presses even when the press belongs to another widget.
        if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)))
            && !cursor.position().is_some_and(|point| layout.bounds().contains(point)) {
            shell.publish(self.on_outside.clone());
            return;
        }
        if matches!(event, Event::Keyboard(keyboard::Event::KeyPressed { key: keyboard::Key::Named(keyboard::key::Named::Escape), .. })) {
            shell.publish(self.on_escape.clone());
            shell.capture_event();
            return;
        }
        self.content.as_widget_mut().update(&mut tree.children[0], event, layout.child(0), cursor, renderer, clipboard, shell, viewport);
    }
    fn mouse_interaction(&self, tree: &Tree, layout: Layout<'_>, cursor: Cursor, viewport: &Rectangle, renderer: &iced::Renderer) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(&tree.children[0], layout.child(0), cursor, viewport, renderer)
    }
    fn overlay<'b>(&'b mut self, tree: &'b mut Tree, layout: Layout<'b>, renderer: &iced::Renderer, viewport: &Rectangle, translation: Vector) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(&mut tree.children[0], layout.child(0), renderer, viewport, translation)
    }
}
impl<'a, Message: Clone + 'a> From<RenameField<'a, Message>> for Element<'a, Message> {
    fn from(widget: RenameField<'a, Message>) -> Self { Element::new(widget) }
}
