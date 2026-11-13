//! Shared dark application palette and fixed-geometry pressable action widgets.
//!
//! Native egui buttons retain input, focus and accessibility ownership. Custom
//! painting changes only their appearance; no worker or guest operation runs here.

use eframe::egui::{
    self, Color32, Response, RichText, Shape, Stroke, StrokeKind, Ui, Vec2, Widget,
};


/// Breeze-style blue used for ordinary action borders, hover tint and focus.
pub(crate) const ACCENT: Color32 = Color32::from_rgb(0x3D, 0xAE, 0xE9);


/// Main application panel background from the reference demonstration.
const BACKGROUND: Color32 = Color32::from_rgb(0x1E, 0x21, 0x24);


/// Window and resting button fill from the reference demonstration.
const PANEL: Color32 = Color32::from_rgb(0x28, 0x2C, 0x30);


/// Shared foreground colour from the reference demonstration.
const TEXT: Color32 = Color32::from_rgb(0xE6, 0xE8, 0xEA);


/// Logical-point padding around action and combo-box labels.
const BUTTON_PADDING: Vec2 = Vec2::new(18.0, 8.0);


/// Hover-transition duration in seconds, matching the reference's 120 ms tint.
const HOVER_SECONDS: f32 = 0.12;


/// Apply the reference dark theme to panels, windows, menus and ordinary controls.
///
/// Explicit application colours, including failure and activity indicators, remain
/// owned by their existing presentations. This changes no worker configuration.
pub(crate) fn apply_dark_theme(context: &egui::Context) {
    context.set_theme(egui::Theme::Dark);
    context.style_mut_of(egui::Theme::Dark, |style| {
        style.visuals.panel_fill = BACKGROUND;
        style.visuals.window_fill = PANEL;
        style.visuals.extreme_bg_color = BACKGROUND;
        style.visuals.faint_bg_color = PANEL;
        style.visuals.override_text_color = Some(TEXT);
        style.visuals.hyperlink_color = ACCENT;
        style.visuals.selection.bg_fill = ACCENT;
        for visuals in [
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            visuals.bg_fill = PANEL;
            visuals.weak_bg_fill = PANEL;
            visuals.corner_radius = 6.into();
            visuals.expansion = 0.0;
        }
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
        style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        style.spacing.button_padding = BUTTON_PADDING;
        style.spacing.combo_width = 220.0;
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
        style.animation_time = HOVER_SECONDS;
    });
}


/// Native button interaction with the reference's animated, stationary appearance.
///
/// Use `Ui::add_enabled` for existing action gates. Padding and font are identical
/// in every state, including disabled, selected, pointer-down and keyboard focus.
pub(crate) struct PressableButton<'a> {
    /// Visible and accessible native button label.
    label: &'a str,
    /// Semantic border and hover colour; ordinary actions use the shared blue.
    accent: Color32,
    /// Optional toggle semantics, absent for ordinary one-shot action buttons.
    selected: Option<bool>,
}


impl<'a> PressableButton<'a> {
    /// Create an ordinary blue-accented action without toggle semantics.
    pub(crate) fn new(label: &'a str) -> Self {
        Self {
            label,
            accent: ACCENT,
            selected: None,
        }
    }


    /// Retain a meaningful action or warning colour without a bright solid fill.
    pub(crate) fn accent(mut self, accent: Color32) -> Self {
        self.accent = accent;
        self
    }


    /// Mark a true toggle with its current accessible selected state.
    pub(crate) fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }


    /// Measure fixed logical-point bounds so a wrapping parent can keep a row together.
    ///
    /// This lays out only the label and never allocates space or registers input.
    pub(crate) fn desired_size(&self, ui: &Ui) -> Vec2 {
        let label = ui.painter().layout_no_wrap(
            self.label.to_owned(),
            egui::FontId::proportional(15.0),
            TEXT,
        );
        let size = label.size() + BUTTON_PADDING * 2.0;
        Vec2::new(size.x, size.y.max(ui.spacing().interact_size.y))
    }
}


impl Widget for PressableButton<'_> {
    /// Delegate interaction to egui and paint fixed bounds around the native label.
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = self.desired_size(ui);
        let background = ui.painter().add(Shape::Noop);
        let previous_style = ui.style().clone();
        let style = ui.style_mut();
        style.spacing.button_padding = BUTTON_PADDING;
        for visuals in [
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.noninteractive,
        ] {
            visuals.expansion = 0.0;
            visuals.bg_stroke = Stroke::NONE;
        }
        let mut button = egui::Button::new(RichText::new(self.label).size(15.0).color(TEXT))
            .frame(true)
            .min_size(desired_size)
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .wrap_mode(egui::TextWrapMode::Extend);
        if let Some(selected) = self.selected {
            button = button.selected(selected);
        }
        let response = ui.add(button);
        ui.set_style(previous_style);

        let hover = ui.ctx().animate_bool_with_time(
            response.id.with("pressable-hover"),
            response.enabled() && response.hovered(),
            HOVER_SECONDS,
        );
        let hovered_fill = linear_blend_colour(PANEL, self.accent, 0.25 * hover);
        let keyboard_pressed = response.has_focus()
            && ui.input(|input| {
                input.key_down(egui::Key::Enter) || input.key_down(egui::Key::Space)
            });
        let pressed = response.enabled()
            && (response.is_pointer_button_down_on()
                || keyboard_pressed
                || self.selected == Some(true));
        let fill = if pressed {
            linear_blend_colour(hovered_fill, Color32::BLACK, 0.25)
        } else {
            hovered_fill
        };
        ui.painter()
            .set(background, Shape::rect_filled(response.rect, 6, fill));
        ui.painter().rect_stroke(
            response.rect,
            6,
            Stroke::new(1.0, self.accent.gamma_multiply(0.6)),
            StrokeKind::Inside,
        );
        if response.enabled() && response.has_focus() {
            ui.painter().rect_stroke(
                response.rect,
                6,
                Stroke::new(2.0, self.accent),
                StrokeKind::Inside,
            );
        }
        response
    }
}


/// Add an ordinary pressable action; parent UI enablement still controls input.
pub(crate) fn press_able_button(ui: &mut Ui, label: &str) -> Response {
    ui.add(PressableButton::new(label))
}


/// Blend RGB channels linearly with a factor clamped to the inclusive unit range.
fn linear_blend_colour(from: Color32, to: Color32, factor: f32) -> Color32 {
    let factor = factor.clamp(0.0, 1.0);
    let mix =
        |from: u8, to: u8| (f32::from(from) + (f32::from(to) - f32::from(from)) * factor) as u8;
    Color32::from_rgb(
        mix(from.r(), to.r()),
        mix(from.g(), to.g()),
        mix(from.b(), to.b()),
    )
}


#[cfg(test)]
mod tests {
    //! Exercise native interaction and export optional actual egui rendering evidence.

    use super::*;
    use egui::{Event, FullOutput, Key, Modifiers, PointerButton, Pos2, RawInput, Rect};


    /// Draw one actual button with reproducible input, returning its response and shapes.
    fn button_frame(
        context: &egui::Context,
        time: f64,
        enabled: bool,
        events: Vec<Event>,
    ) -> (Response, FullOutput) {
        let mut response = None;
        let mut output = context.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(840.0, 600.0))),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ui| response = Some(ui.add_enabled(enabled, PressableButton::new("Action"))),
        );
        output.textures_delta.clear();
        (response.expect("button drawn"), output)
    }


    /// Extract the actual opaque button background without relying on paint-list indices.
    fn background(output: &FullOutput, bounds: Rect) -> Color32 {
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                Shape::Rect(rect) if rect.rect == bounds && rect.fill.a() > 0 => Some(rect.fill),
                _ => None,
            })
            .expect("button background painted")
    }


    /// Construct one primary-button transition at a known actual control centre.
    fn pointer(pos: Pos2, pressed: bool) -> Event {
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }
    }


    /// Pointer hover/down/release changes paint while keeping layout and native click metadata.
    #[test]
    fn pressable_pointer_states_keep_geometry_and_native_click() {
        let context = egui::Context::default();
        apply_dark_theme(&context);
        let (idle, idle_output) = button_frame(&context, 0.0, true, vec![]);
        assert_eq!(background(&idle_output, idle.rect), PANEL);
        let centre = idle.rect.center();
        button_frame(&context, 0.2, true, vec![Event::PointerMoved(centre)]);
        let (hovered, hover_output) = button_frame(&context, 0.4, true, vec![]);
        let hover_fill = background(&hover_output, hovered.rect);
        assert_ne!(hover_fill, PANEL);
        let (pressed, down_output) = button_frame(&context, 0.5, true, vec![pointer(centre, true)]);
        let down_fill = background(&down_output, pressed.rect);
        assert!(pressed.is_pointer_button_down_on());
        assert!(!pressed.clicked());
        assert!(down_fill.r() < hover_fill.r() && down_fill.g() < hover_fill.g());
        let (released, release_output) =
            button_frame(&context, 0.6, true, vec![pointer(centre, false)]);
        assert!(released.clicked());
        assert!(release_output.platform_output.events.iter().any(|event| {
            matches!(event, egui::output::OutputEvent::Clicked(info)
                if info.typ == egui::WidgetType::Button
                    && info.enabled && info.label.as_deref() == Some("Action")
                    && info.selected.is_none())
        }));
        for state in [hovered, pressed, released] {
            assert_eq!(state.rect, idle.rect);
        }
    }


    /// Focus has a visible ring and Space still activates the native button; disabled input cannot.
    #[test]
    fn pressable_focus_keyboard_and_disabled_gates_remain_native() {
        let context = egui::Context::default();
        apply_dark_theme(&context);
        let (idle, _) = button_frame(&context, 0.0, true, vec![]);
        context.memory_mut(|memory| memory.request_focus(idle.id));
        let (focused, output) = button_frame(&context, 0.1, true, vec![]);
        assert!(focused.has_focus());
        assert_eq!(focused.rect, idle.rect);
        assert!(output.shapes.iter().any(|shape| {
            matches!(&shape.shape, Shape::Rect(rect)
                if rect.rect == idle.rect && rect.stroke == Stroke::new(2.0, ACCENT))
        }));
        let key = Event::Key {
            key: Key::Space,
            physical_key: Some(Key::Space),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        let (activated, _) = button_frame(&context, 0.2, true, vec![key.clone()]);
        assert!(activated.clicked());
        let (disabled, _) = button_frame(&context, 0.3, false, vec![key]);
        assert!(!disabled.enabled());
        assert!(!disabled.clicked());
        assert!(!disabled.has_focus());
        assert_eq!(disabled.rect, idle.rect);
        let centre = disabled.rect.center();
        let (down, _) = button_frame(&context, 0.4, false, vec![pointer(centre, true)]);
        let (up, _) = button_frame(&context, 0.5, false, vec![pointer(centre, false)]);
        assert!(!down.is_pointer_button_down_on());
        assert!(!up.clicked());
    }


    /// Render shared action widgets and a native combo without starting the application or worker.
    ///
    /// When requested, JSON includes tessellated geometry and every font-texture update
    /// needed to rasterise the frame externally. No output directory means no files.
    #[test]
    fn render_pressable_theme_evidence() {
        let evidence = std::env::var_os("QMP_UI_STYLE_EVIDENCE_DIR").map(std::path::PathBuf::from);
        if let Some(directory) = &evidence {
            std::fs::create_dir_all(directory).expect("create optional style-evidence directory");
        }
        for width in [840.0, 1280.0] {
            let context = egui::Context::default();
            apply_dark_theme(&context);
            let mut button = None;
            let mut textures = Vec::new();
            for (index, state) in [
                "idle",
                "hover",
                "hover-settled",
                "down",
                "focus",
                "disabled",
            ]
            .into_iter()
            .enumerate()
            {
                let centre = button
                    .as_ref()
                    .map_or(Pos2::ZERO, |response: &Response| response.rect.center());
                if state == "focus" {
                    context.memory_mut(|memory| {
                        memory.request_focus(button.as_ref().expect("previous button").id)
                    });
                }
                let events = match state {
                    "hover" => vec![Event::PointerMoved(centre)],
                    "down" => vec![pointer(centre, true)],
                    "focus" => vec![pointer(centre, false), Event::PointerGone],
                    _ => vec![],
                };
                let enabled = state != "disabled";
                let mut output = context.run_ui(
                    RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            Pos2::ZERO,
                            egui::vec2(width, 240.0),
                        )),
                        time: Some(index as f64 * 0.2),
                        events,
                        ..Default::default()
                    },
                    |ui| {
                        egui::CentralPanel::default().show(ui, |ui| {
                            ui.heading("Application controls");
                            ui.horizontal_wrapped(|ui| {
                                button = Some(
                                    ui.add_enabled(enabled, PressableButton::new("CALCULATE")),
                                );
                                press_able_button(ui, "Pause");
                                ui.add(PressableButton::new("Cancel").accent(Color32::LIGHT_RED));
                                egui::ComboBox::from_id_salt("style-evidence-selection")
                                    .selected_text("Maximum card matches")
                                    .show_ui(ui, |ui| {
                                        ui.label("Maximum card matches");
                                    });
                            });
                            ui.label("Fixed bounds · keyboard focus · native enabled state");
                        });
                    },
                );
                assert!(button.as_ref().expect("button drawn").rect.right() <= width);
                let Some(directory) = &evidence else {
                    output.textures_delta.clear();
                    continue;
                };
                for (texture, deltas) in &output.textures_delta.set {
                    for delta in deltas {
                        let egui::ImageData::Color(image) = &delta.image;
                        textures.push(serde_json::json!({
                            "id":texture_key(*texture), "size":image.size, "pos":delta.pos,
                            "rgba":image.pixels.iter().flat_map(|pixel| pixel.to_array()).collect::<Vec<_>>()
                        }));
                    }
                }
                output.textures_delta.clear();
                let meshes = context.tessellate(output.shapes, output.pixels_per_point)
                    .into_iter().map(|primitive| {
                        let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive else { panic!("unexpected paint callback"); };
                        serde_json::json!({
                            "clip":[primitive.clip_rect.min.x,primitive.clip_rect.min.y,primitive.clip_rect.max.x,primitive.clip_rect.max.y],
                            "texture":texture_key(mesh.texture_id), "indices":mesh.indices,
                            "vertices":mesh.vertices.iter().map(|vertex| serde_json::json!({
                                "pos":[vertex.pos.x,vertex.pos.y],"uv":[vertex.uv.x,vertex.uv.y],"rgba":vertex.color.to_array()
                            })).collect::<Vec<_>>()
                        })
                    }).collect::<Vec<_>>();
                let artifact = serde_json::json!({
                    "width":width,"height":240,"pixels_per_point":output.pixels_per_point,
                    "state":state,"textures":textures,"meshes":meshes
                });
                std::fs::write(
                    directory.join(format!("style-{width:.0}-{state}.json")),
                    serde_json::to_vec(&artifact).expect("serialise style evidence"),
                )
                .expect("write optional style evidence");
            }
        }
    }


    /// Preserve managed/user texture identity in optional rendering evidence.
    fn texture_key(texture: egui::TextureId) -> String {
        match texture {
            egui::TextureId::Managed(id) => format!("managed-{id}"),
            egui::TextureId::User(id) => format!("user-{id}"),
        }
    }
}
