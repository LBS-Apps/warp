use warpui_core::event::ModifiersState;

use super::grid::grid_handler::GridHandler;
use super::indexing::Point;

#[derive(Debug, Copy, Clone)]
pub enum MouseButton {
    Left,
    Right,
    Wheel,
    LeftDrag,
    Move, // Used for mouse hover events (when cursor is moving)
}
#[derive(Debug, Copy, Clone)]
pub enum MouseAction {
    Pressed,
    Released,
    Scrolled { delta: i32 },
}

#[derive(Debug, Copy, Clone)]
pub struct MouseState {
    button: MouseButton,
    action: MouseAction,
    point: Option<Point>,
    modifiers: ModifiersState,
}

impl MouseState {
    pub fn new(button: MouseButton, action: MouseAction, modifiers: ModifiersState) -> Self {
        Self {
            button,
            action,
            point: None,
            modifiers,
        }
    }

    pub fn set_point(mut self, p: Point) -> Self {
        self.point = Some(p);
        self
    }

    /// Sets the point for a pointer over the visual (on-screen) cell `visual` of `grid`. Programs
    /// that read mouse reports address cells in logical order, but on a row the renderer
    /// bidi-reorders the glyph under the pointer belongs to a different logical cell; reporting
    /// the on-screen column would put the program on the mirror-image character of the RTL run.
    pub fn set_point_under_pointer(self, visual: Point, grid: &GridHandler) -> Self {
        self.set_point(grid.bidi_visual_to_logical(visual))
    }

    pub fn button(&self) -> &MouseButton {
        &self.button
    }
    pub fn action(&self) -> &MouseAction {
        &self.action
    }

    pub fn maybe_point(&self) -> Option<Point> {
        self.point
    }

    pub fn modifiers(&self) -> &ModifiersState {
        &self.modifiers
    }
}
