use iced::advanced::{
    Clipboard, Layout, Overlay, Shell, Widget, layout,
    mouse::Cursor,
    overlay, renderer,
    widget::{Operation, Tree},
};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, keyboard, mouse};

#[derive(Debug, Clone, Copy)]
pub enum Placement {
    TopRight,
    Center,
}

fn popup_origin(placement: Placement, window: Size, popup: Size) -> Point {
    let margin = 18.0;
    match placement {
        Placement::TopRight => Point::new(
            (window.width - popup.width - margin).max(0.0),
            margin.min((window.height - popup.height).max(0.0)),
        ),
        Placement::Center => Point::new(
            ((window.width - popup.width) / 2.0).max(0.0),
            ((window.height - popup.height) / 2.0).max(0.0),
        ),
    }
}

pub struct AppOverlay<'a, Message> {
    content: Element<'a, Message>,
    toast: Option<Element<'a, Message>>,
    modal: Option<Element<'a, Message>>,
    close_modal: Message,
}

impl<'a, Message> AppOverlay<'a, Message> {
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        toast: Option<Element<'a, Message>>,
        modal: Option<Element<'a, Message>>,
        close_modal: Message,
    ) -> Self {
        Self {
            content: content.into(),
            toast,
            modal,
            close_modal,
        }
    }
}

impl<Message: Clone> Widget<Message, iced::Theme, iced::Renderer> for AppOverlay<'_, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn children(&self) -> Vec<Tree> {
        let mut children = vec![Tree::new(&self.content)];
        if let Some(toast) = &self.toast {
            children.push(Tree::new(toast));
        }
        if let Some(modal) = &self.modal {
            children.push(Tree::new(modal));
        }
        children
    }

    fn diff(&self, tree: &mut Tree) {
        let mut children = vec![&self.content];
        if let Some(toast) = &self.toast {
            children.push(toast);
        }
        if let Some(modal) = &self.modal {
            children.push(modal);
        }
        tree.diff_children(&children);
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

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.child(0),
                renderer,
                operation,
            );
        });
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
        // modal events are captured by the overlay before reaching this base widget
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
        let mut layers = Vec::new();
        let mut trees = tree.children.iter_mut();
        let content_tree = trees.next().expect("content tree");

        if let Some(child_overlay) = self.content.as_widget_mut().overlay(
            content_tree,
            layout.child(0),
            renderer,
            viewport,
            translation,
        ) {
            layers.push(child_overlay);
        }

        if let Some(toast) = self.toast.as_mut() {
            let toast_tree = trees.next().expect("toast tree");
            layers.push(overlay::Element::new(Box::new(FloatingOverlay {
                content: toast,
                tree: toast_tree,
                placement: Placement::TopRight,
                dismiss: None,
                modal: false,
            })));
        }
        if let Some(modal) = self.modal.as_mut() {
            let modal_tree = trees.next().expect("modal tree");
            layers.push(overlay::Element::new(Box::new(FloatingOverlay {
                content: modal,
                tree: modal_tree,
                placement: Placement::Center,
                dismiss: Some(self.close_modal.clone()),
                modal: true,
            })));
        }

        match layers.len() {
            0 => None,
            1 => Some(layers.remove(0)),
            _ => Some(overlay::Group::with_children(layers).overlay()),
        }
    }
}

impl<'a, Message: Clone + 'a> From<AppOverlay<'a, Message>> for Element<'a, Message> {
    fn from(widget: AppOverlay<'a, Message>) -> Self {
        Element::new(widget)
    }
}

struct FloatingOverlay<'a, 'b, Message> {
    content: &'a mut Element<'b, Message>,
    tree: &'a mut Tree,
    placement: Placement,
    dismiss: Option<Message>,
    modal: bool,
}

impl<Message: Clone> Overlay<Message, iced::Theme, iced::Renderer>
    for FloatingOverlay<'_, '_, Message>
{
    fn layout(&mut self, renderer: &iced::Renderer, window: Size) -> layout::Node {
        let max_width = match self.placement {
            Placement::TopRight => window.width.min(375.0),
            Placement::Center => window.width.min(710.0),
        };
        let limits = layout::Limits::new(Size::ZERO, Size::new(max_width, window.height.max(0.0)));
        let child = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let size = child.size();
        layout::Node::with_children(size, vec![child]).move_to(popup_origin(
            self.placement,
            window,
            size,
        ))
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
        if let (
            Some(dismiss),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }),
        ) = (&self.dismiss, event)
        {
            shell.publish(dismiss.clone());
            shell.capture_event();
            return;
        }
        let bounds = layout.bounds();
        let inside = cursor.position().is_some_and(|p| bounds.contains(p));
        let click = matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_)));

        if self.modal && click && !inside {
            if let Some(dismiss) = &self.dismiss {
                shell.publish(dismiss.clone());
            }
            shell.capture_event();
            return;
        }
        if inside || !click {
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
        // prevent clicking underlying controls through a non-opaque popup
        if inside && click {
            shell.capture_event();
        }
        if self.modal && matches!(event, Event::Keyboard(_)) {
            shell.capture_event();
        }
    }

    fn index(&self) -> f32 {
        if self.modal { 20.0 } else { 10.0 }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlay_position_is_clamped_to_small_windows() {
        let p = popup_origin(
            Placement::TopRight,
            Size::new(100.0, 70.0),
            Size::new(200.0, 100.0),
        );
        assert_eq!(p.x, 0.0);
        assert_eq!(p.y, 0.0);
    }
}
