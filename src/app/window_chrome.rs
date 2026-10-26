//! Shared in-application window chrome using public egui 0.36.2 APIs.
//!
//! The application retains displayed data and visibility. This helper owns only
//! session presentation state; it does not access a worker, filesystem or guest.

use eframe::egui;
use eframe::egui::{
    Align2, Color32, Context, FontId, Id, InnerResponse, Pos2, Rect, Response, RichText, Sense,
    Stroke, StrokeKind, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType, Window, WindowDrag,
};

/// Horizontal title padding in egui points.
const TITLE_PADDING_X: i8 = 8;
/// Vertical title padding, matched by the native title frame, in egui points.
const TITLE_PADDING_Y: i8 = 5;
/// Preferred square control width in egui points.
const CONTROL_WIDTH: f32 = 26.0;
/// Shared visible outline width in egui points.
const BORDER_WIDTH: f32 = 1.0;
/// Space reserved for a readable title before the three controls, in points.
const MINIMUM_TITLE_WIDTH: f32 = 110.0;

/// Session-owned presentation state, independent of each window's retained data.
#[derive(Default)]
pub(super) struct WindowChrome {
    /// Last ordinary expanded outer rectangle; collapsed movement updates its origin.
    normal_rect: Option<Rect>,
    /// Whether the window occupies the application's safe content area.
    maximised: bool,
    /// Whether the body is hidden; this does not close the window.
    collapsed: bool,
    /// Geometry to apply once after restoring or expanding an ordinary window.
    pending_rect: Option<Rect>,
}

/// Appearance and initial outer dimensions, all measured in egui points.
#[derive(Clone, Copy)]
pub(super) struct WindowAppearance {
    /// Stable identity shared by the native Window and its own chrome widgets.
    pub(super) id: Id,
    /// Human-readable title; its text does not determine the stable identity.
    pub(super) title: &'static str,
    /// Contrasting titlebar colour; controls and outline are white.
    pub(super) colour: Color32,
    /// Initial outer dimensions, including the native title and border.
    pub(super) default_size: Vec2,
    /// Ordinary minimum outer dimensions; maximised/collapsed modes override them.
    pub(super) min_size: Vec2,
}

impl WindowChrome {
    /// Show a native floating window with a custom painted title and three controls.
    ///
    /// `open` controls only visibility. Closing never clears application data.
    /// `add_contents` is called only while expanded. The returned inner value is
    /// absent while collapsed, and the entire response is absent while closed.
    /// Geometry changes from title controls take effect on the following frame.
    pub(super) fn show<R>(
        &mut self,
        context: &Context,
        appearance: WindowAppearance,
        open: &mut bool,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<Option<R>>> {
        if !*open {
            return None;
        }
        let workspace = context.content_rect();
        if !workspace.is_finite() || workspace.width() <= 0.0 || workspace.height() <= 0.0 {
            return None;
        }

        let style = context.global_style();
        let font: FontId = egui::TextStyle::Heading.resolve(&style);
        let title_height =
            context.fonts_mut(|fonts| fonts.row_height(&font)) + 2.0 * f32::from(TITLE_PADDING_Y);
        let collapsed_height = title_height + 2.0 * BORDER_WIDTH;
        let minimum_width = MINIMUM_TITLE_WIDTH + 3.0 * CONTROL_WIDTH;
        let frame = egui::Frame::window(&style)
            .inner_margin(egui::Margin::same(8))
            .outer_margin(0.0)
            .corner_radius(0)
            .stroke(Stroke::new(BORDER_WIDTH, Color32::WHITE));
        let mut window = Window::new(RichText::new(appearance.title).color(Color32::WHITE))
            .id(appearance.id)
            .collapsible(false)
            .default_open(true)
            .drag_area(WindowDrag::TitleBar)
            .frame(if self.collapsed {
                frame.inner_margin(0.0)
            } else {
                frame
            })
            .title_frame(
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(TITLE_PADDING_X, TITLE_PADDING_Y))
                    .fill(appearance.colour),
            )
            .default_size(appearance.default_size)
            .min_size(
                appearance
                    .min_size
                    .max(Vec2::new(minimum_width, collapsed_height)),
            )
            .constrain_to(workspace);

        let collapsed = self.collapsed;
        if self.maximised {
            let size = if collapsed {
                Vec2::new(workspace.width(), collapsed_height.min(workspace.height()))
            } else {
                workspace.size()
            };
            window = window.fixed_rect(Rect::from_min_size(workspace.min, size));
        } else if collapsed {
            let width = self
                .normal_rect
                .map_or(appearance.default_size.x, |rect| rect.width());
            window = window
                .fixed_size(Vec2::new(width.min(workspace.width()), collapsed_height))
                .movable(true)
                .resizable(false);
        } else if let Some(rect) = self.pending_rect.take() {
            // Area constrains its cached maximised size before laying out the smaller body.
            // Clamp the saved geometry ourselves for this one restoration frame.
            let size = rect.size().min(workspace.size());
            let position = rect.min.clamp(workspace.min, workspace.max - size);
            window = window
                .fixed_rect(Rect::from_min_size(position, size))
                .constrain(false);
        } else {
            window = window.resizable(true);
        }

        let shown = window.show(context, |ui| {
            if collapsed {
                None
            } else {
                // Window::fixed_size constrains layout; this also makes the body occupy it.
                // Each caller retains its existing content/scrolling policy.
                ui.set_min_size(ui.available_size());
                Some(add_contents(ui))
            }
        })?;
        let outer = shown.response.rect;
        if !self.maximised {
            if collapsed {
                if let Some(rect) = self.normal_rect.as_mut() {
                    // Dragging a collapsed title moves its future expanded origin only.
                    *rect = Rect::from_min_size(outer.min, rect.size());
                }
            } else {
                self.normal_rect = Some(outer);
            }
        }

        let command = paint_title(
            context,
            &shown.response,
            appearance,
            &font,
            title_height,
            self,
        );
        match command {
            Some(Command::Collapse) => {
                self.collapsed = !self.collapsed;
                if !self.collapsed && !self.maximised {
                    self.pending_rect = self.normal_rect;
                }
            }
            Some(Command::Maximise) => {
                self.maximised = !self.maximised;
                self.collapsed = false;
                if !self.maximised {
                    self.pending_rect = self.normal_rect;
                }
            }
            Some(Command::Close) => *open = false,
            None => {}
        }
        if command.is_some() {
            context.request_repaint();
        }
        Some(InnerResponse {
            inner: shown.inner.flatten(),
            response: shown.response,
        })
    }
}

/// A single presentation operation accepted from this frame's title controls.
#[derive(Clone, Copy)]
enum Command {
    /// Hide or show the body without changing visibility or its retained data.
    Collapse,
    /// Fill the safe application content area or restore ordinary outer geometry.
    Maximise,
    /// Change only the caller's visibility flag.
    Close,
}

/// Paint a title and clickable controls on the native window's existing layer.
///
/// A separate Area would introduce another window-sized Z-order owner. The root
/// Ui here shares the Window response's layer and only covers its title rectangle.
fn paint_title(
    context: &Context,
    response: &Response,
    appearance: WindowAppearance,
    font: &FontId,
    title_height: f32,
    state: &WindowChrome,
) -> Option<Command> {
    let inner = response.rect.shrink(BORDER_WIDTH);
    let header = Rect::from_min_max(
        inner.min,
        Pos2::new(inner.max.x, (inner.min.y + title_height).min(inner.max.y)),
    );
    if !header.is_positive() {
        return None;
    }
    let mut ui = Ui::new(
        context.clone(),
        appearance.id.with("window-chrome"),
        UiBuilder::new()
            .layer_id(response.layer_id)
            .max_rect(header),
    );
    ui.set_clip_rect(header);
    ui.painter().rect_filled(header, 0, appearance.colour);
    // Leave native top/right edge and top-right corner resize targets unobstructed.
    let interaction = &ui.style().interaction;
    let controls_right = (header.max.x - interaction.resize_grab_radius_corner).max(header.min.x);
    let controls_top = (header.min.y + interaction.resize_grab_radius_side).min(header.max.y);
    let button_width = CONTROL_WIDTH.min((controls_right - header.min.x - 2.0).max(0.0) / 3.0);
    let controls_left = controls_right - 3.0 * button_width;
    let text_rect = Rect::from_min_max(
        header.min + Vec2::new(f32::from(TITLE_PADDING_X), 0.0),
        Pos2::new(
            (controls_left - f32::from(TITLE_PADDING_X)).max(header.min.x),
            header.max.y,
        ),
    );
    if text_rect.is_positive() {
        ui.painter().with_clip_rect(text_rect).text(
            Pos2::new(text_rect.min.x, text_rect.center().y),
            Align2::LEFT_CENTER,
            appearance.title,
            font.clone(),
            Color32::WHITE,
        );
    }

    let mut accepted = None;
    let controls = [
        (
            0_u8,
            Command::Collapse,
            if state.collapsed {
                "Expand"
            } else {
                "Collapse"
            },
        ),
        (
            1_u8,
            Command::Maximise,
            if state.maximised {
                "Restore"
            } else {
                "Maximise"
            },
        ),
        (2_u8, Command::Close, "Close window"),
    ];
    for (index, command, label) in controls {
        let left = controls_left + f32::from(index) * button_width;
        let rect = Rect::from_min_max(
            Pos2::new(left, controls_top),
            Pos2::new(left + button_width, header.max.y),
        );
        // Absorb dragging over controls so it cannot activate native titlebar movement.
        let button = ui.interact(
            rect,
            appearance.id.with(("window-control", index)),
            Sense::click_and_drag(),
        );
        button.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
        if button.hovered() || button.is_pointer_button_down_on() {
            ui.painter()
                .rect_filled(rect.shrink(1.0), 2, Color32::from_white_alpha(35));
        }
        paint_icon(&ui, rect.shrink(7.0), command, state, appearance.colour);
        if button.clicked() {
            accepted = Some(command);
        }
        button.on_hover_text(label);
    }
    context.layer_painter(response.layer_id).rect_stroke(
        response.rect,
        0,
        Stroke::new(BORDER_WIDTH, Color32::WHITE),
        StrokeKind::Inside,
    );
    accepted
}

/// Paint dependency-free white line icons inside a bounded control rectangle.
fn paint_icon(ui: &Ui, rect: Rect, command: Command, state: &WindowChrome, title_colour: Color32) {
    if !rect.is_positive() {
        return;
    }
    let stroke = Stroke::new(1.5, Color32::WHITE);
    let painter = ui.painter();
    match command {
        Command::Collapse if state.collapsed => {
            let center = rect.center();
            painter.line_segment([rect.left_center(), center + Vec2::new(0.0, 3.0)], stroke);
            painter.line_segment([center + Vec2::new(0.0, 3.0), rect.right_center()], stroke);
        }
        Command::Collapse => {
            painter.line_segment([rect.left_center(), rect.right_center()], stroke);
        }
        Command::Maximise if state.maximised => {
            let rear = Rect::from_min_max(
                rect.min + Vec2::new(3.0, 0.0),
                rect.max - Vec2::new(0.0, 3.0),
            );
            let front = Rect::from_min_max(
                rect.min + Vec2::new(0.0, 3.0),
                rect.max - Vec2::new(3.0, 0.0),
            );
            painter.rect_stroke(rear, 0, stroke, StrokeKind::Inside);
            painter.rect_filled(front, 0, title_colour);
            painter.rect_stroke(front, 0, stroke, StrokeKind::Inside);
        }
        Command::Maximise => {
            painter.rect_stroke(rect, 0, stroke, StrokeKind::Inside);
        }
        Command::Close => {
            painter.line_segment([rect.left_top(), rect.right_bottom()], stroke);
            painter.line_segment([rect.right_top(), rect.left_bottom()], stroke);
        }
    }
}
