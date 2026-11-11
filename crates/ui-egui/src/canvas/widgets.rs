use super::draw::draw_list;
use super::grid::draw_grid;
use super::{CadApp, Color32, Pos2, Rect, Rgb, Shape, Stroke, Tokens, Xf, pos2, vec2};

pub(super) fn tile_rects(app: &CadApp, full: Rect) -> Vec<(Rect, autocraft_engine::ModelTile)> {
    let x = |f: f64| full.left() + full.width() * f.clamp(0.0, 1.0) as f32;
    let y = |f: f64| full.bottom() - full.height() * f.clamp(0.0, 1.0) as f32;
    app.session
        .model_tiles()
        .into_iter()
        .map(|t| (Rect::from_min_max(pos2(x(t.min.x), y(t.max.y)), pos2(x(t.max.x), y(t.min.y))).shrink(1.0), t))
        .collect()
}

/// The world ↔ screen transform of a model tile drawn in `r`.
pub(super) fn tile_xf(r: Rect, tile: &autocraft_engine::ModelTile) -> Xf {
    Xf { rect: r, center: tile.view.center, scale: f64::from(r.height()) / tile.view.height.max(1e-12), twist: tile.view.twist }
}

/// The tiles other than the current one (each with its own view, clipped to it), then the
/// tile borders: thin, and heavy around the current tile.
pub(super) fn draw_tiles(app: &CadApp, ui: &egui::Ui, tiles: &[(Rect, autocraft_engine::ModelTile)]) {
    let t = Tokens::get();
    let bg = Rgb(t.canvas.r(), t.canvas.g(), t.canvas.b());
    for (r, tile) in tiles.iter().filter(|(r, tile)| !tile.active && r.width() >= 1.0 && r.height() >= 1.0) {
        let p = ui.painter_at(*r);
        let xf = tile_xf(*r, tile);
        p.rect_filled(*r, 0.0, t.canvas);
        draw_grid(app, &p, &xf);
        if let Some(list) = &app.canvas.list {
            draw_list(&p, &xf, list, bg, app.session.settings.lwdisplay);
        }
        viewport_label(&p, *r);
    }
    let p = ui.painter();
    for (r, tile) in tiles {
        let (w, c) = if tile.active { (2.0, t.accent) } else { (1.0, t.border) };
        p.rect_stroke(*r, 0.0, Stroke::new(w, c), egui::StrokeKind::Outside);
    }
}

/// A small padlock, drawn in code: body plus shackle.
pub(super) fn draw_lock(p: &egui::Painter, c: Pos2, color: Color32) {
    p.rect_filled(Rect::from_center_size(c + vec2(0.0, 1.5), vec2(7.0, 5.0)), 1.0, color);
    p.add(Shape::line(
        vec![pos2(c.x - 2.0, c.y - 1.0), pos2(c.x - 2.0, c.y - 4.0), pos2(c.x + 2.0, c.y - 4.0), pos2(c.x + 2.0, c.y - 1.0)],
        Stroke::new(1.2, color),
    ));
}

pub(super) fn draw_ucs_icon(p: &egui::Painter, rect: Rect) {
    let t = Tokens::get();
    let o = pos2(rect.left() + 36.0, rect.bottom() - 34.0);
    let st = Stroke::new(1.0, t.canvas_ink);
    p.line_segment([o, o + vec2(60.0, 0.0)], st);
    p.line_segment([o, o + vec2(0.0, -60.0)], st);
    p.rect_stroke(Rect::from_center_size(o, vec2(9.0, 9.0)), 0.0, st, egui::StrokeKind::Middle);
    let f = egui::FontId::proportional(13.0);
    p.text(o + vec2(70.0, 0.0), egui::Align2::LEFT_CENTER, crate::tl!("X"), f.clone(), t.canvas_ink);
    p.text(o + vec2(0.0, -70.0), egui::Align2::CENTER_BOTTOM, crate::tl!("Y"), f, t.canvas_ink);
}

pub(super) fn viewport_label(p: &egui::Painter, rect: Rect) {
    let t = Tokens::get();
    let at = pos2(rect.left() + 10.0, rect.top() + 8.0);
    p.text(at, egui::Align2::LEFT_TOP, crate::tl!("[-][Top][2D Wireframe]"), crate::theme::small(), t.canvas_ink);
}

/// The viewport controls at the drawing's top-left corner, "[-][Top][2D Wireframe]": each
/// bracket opens its menu (viewport tools and configurations, views, visual styles).
pub(super) fn viewport_controls(app: &mut CadApp, ui: &mut egui::Ui, rect: Rect) {
    let t = Tokens::get();
    let p = ui.painter_at(rect);
    let font = crate::theme::small();
    let mut x = rect.left() + 6.0;
    let y = rect.top() + 5.0;
    let parts = ["-", crate::tl!("Top"), crate::tl!("2D Wireframe")];
    let mut pick: Option<&'static str> = None;
    for (k, part) in parts.into_iter().enumerate() {
        let g = p.layout_no_wrap(format!("[{part}]"), font.clone(), t.canvas_ink);
        let r = Rect::from_min_size(pos2(x, y), g.size());
        let resp = ui.interact(r, ui.scope_id().with(("vpctl", k)), egui::Sense::click());
        let color = if resp.hovered() { t.accent } else { t.canvas_ink };
        let gw = g.size().x;
        p.galley(pos2(x, y), g, color);
        crate::icons::describe_control(ui, &resp, ["Viewport Controls", "View Controls", "Visual Style Controls"][k], None, false);
        egui::Popup::menu(&resp).show(|ui| match k {
            0 => {
                for (cmd, label, on) in [
                    ("ui.toggle.viewcube", "ViewCube", Some(app.ui.show_viewcube)),
                    ("ui.toggle.navbar", "Navigation Bar", Some(app.ui.show_navbar)),
                    ("ui.toggle.ucsicon", "UCS Icon", Some(app.ui.show_ucs_icon)),
                ] {
                    if ui.add(egui::Button::selectable(on.unwrap_or(false), crate::i18n::t(label))).clicked() {
                        pick = Some(cmd);
                        ui.close();
                    }
                }
                ui.separator();
                ui.menu_button(crate::tl!("Viewport Configuration List"), |ui| {
                    for (cmd, label) in
                        [("vports.1", "Single"), ("vports.2", "Two: Vertical"), ("vports.3", "Three: Right"), ("vports.4", "Four: Equal")]
                    {
                        if ui.button(crate::i18n::t(label)).clicked() {
                            pick = Some(cmd);
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button(crate::tl!("Configure...")).clicked() {
                        pick = Some("vports");
                        ui.close();
                    }
                });
            }
            1 => {
                let _ = ui.add(egui::Button::selectable(true, crate::tl!("Top")));
                for v in ["Bottom", "Left", "Right", "Front", "Back", "SW Isometric", "SE Isometric", "NE Isometric", "NW Isometric"] {
                    ui.add_enabled(false, egui::Button::new(crate::i18n::t(v)));
                }
                ui.separator();
                if ui.button(crate::tl!("View Manager...")).clicked() {
                    pick = Some("view");
                    ui.close();
                }
            }
            _ => {
                let _ = ui.add(egui::Button::selectable(true, crate::tl!("2D Wireframe")));
                for v in ["Conceptual", "Hidden", "Realistic", "Shaded", "Shaded with edges", "Shades of Gray", "Sketchy", "Wireframe", "X-Ray"] {
                    ui.add_enabled(false, egui::Button::new(crate::i18n::t(v)));
                }
            }
        });
        x += gw;
    }
    if let Some(cmd) = pick {
        crate::menus::activate(app, cmd);
    }
}

/// The navigation bar under the ViewCube: Pan, Zoom Extents (with the zoom list) and Orbit in a
/// narrow translucent column at the right edge, its menu button at the bottom.
pub(super) fn navigation_bar(app: &mut CadApp, ui: &mut egui::Ui, rect: Rect) {
    use crate::icons::Icon;
    const TOOLS: &[(Icon, &str, &str)] = &[
        (Icon::Orbit, "ui.noop", "Full Navigation Wheel"),
        (Icon::Pan, "pan", "Pan"),
        (Icon::ZoomExtents, "zoom.extents", "Zoom Extents"),
        (Icon::Orbit, "3dorbit", "Orbit"),
    ];
    if !app.ui.show_navbar {
        return;
    }
    let t = Tokens::get();
    let cube = crate::viewcube::Layout::new(rect);
    let (w, cell) = (24.0, 26.0);
    let top = if app.ui.show_viewcube { cube.pill.bottom() + 14.0 } else { rect.top() + 16.0 };
    let bar = Rect::from_min_size(pos2(rect.right() - w - 6.0, top), vec2(w, cell * TOOLS.len() as f32 + 18.0));
    if bar.bottom() > rect.bottom() - 60.0 {
        return;
    }
    // One widget under the whole bar keeps clicks and the wheel off the drawing.
    let _ = ui.interact(bar, ui.scope_id().with("navbar"), egui::Sense::click());
    let p = ui.painter_at(rect);
    p.rect_filled(bar, 2.0, Color32::from_rgba_unmultiplied(0x55, 0x5d, 0x69, 0xb0));
    p.rect_stroke(bar, 2.0, Stroke::new(1.0, Color32::from_rgba_unmultiplied(0x80, 0x88, 0x94, 0x90)), egui::StrokeKind::Inside);
    let mut start = None;
    for (i, (icon, cmd, label)) in TOOLS.iter().enumerate() {
        let br = Rect::from_min_size(pos2(bar.left() + 1.0, bar.top() + 3.0 + i as f32 * cell), vec2(w - 2.0, cell - 2.0));
        let enabled = crate::menus::is_ui_command(cmd) && *cmd != "ui.noop" || autocraft_engine::find_command(cmd).is_some();
        let resp = ui.interact(br, ui.scope_id().with(("navbar", i)), if enabled { egui::Sense::click() } else { egui::Sense::hover() });
        if enabled && resp.hovered() {
            p.rect_filled(br, 2.0, t.control_hover);
        }
        crate::icons::paint(&p, br.shrink(4.0), *icon, !enabled);
        crate::icons::describe_control(ui, &resp, label, None, false);
        let resp = resp.on_hover_text(crate::i18n::t(label));
        if *cmd == "zoom.extents" {
            // The zoom tool's list, from its right-click, as the bar's split button.
            egui::Popup::context_menu(&resp).show(|ui| {
                for (c, l) in [
                    ("zoom.extents", "Zoom Extents"),
                    ("zoom.window", "Zoom Window"),
                    ("zoom.previous", "Zoom Previous"),
                    ("zoom.realtime", "Zoom Realtime"),
                    ("zoom.all", "Zoom All"),
                    ("zoom.in", "Zoom In"),
                    ("zoom.out", "Zoom Out"),
                ] {
                    if ui.button(crate::i18n::t(l)).clicked() {
                        start = Some(c);
                        ui.close();
                    }
                }
            });
        }
        if enabled && resp.clicked() {
            start = Some(*cmd);
        }
    }
    // The bar's menu button: close it.
    let mr = Rect::from_min_size(pos2(bar.left() + 1.0, bar.bottom() - 15.0), vec2(w - 2.0, 13.0));
    let mresp = ui.interact(mr, ui.scope_id().with("navbar_menu"), egui::Sense::click());
    let c = mr.center();
    p.add(egui::Shape::convex_polygon(vec![c + vec2(-3.0, -1.5), c + vec2(3.0, -1.5), c + vec2(0.0, 2.0)], t.text_dim, Stroke::NONE));
    crate::icons::describe_control(ui, &mresp, "Navigation Bar menu", None, false);
    egui::Popup::menu(&mresp).show(|ui| {
        if ui.button(crate::tl!("Close Navigation Bar")).clicked() {
            start = Some("ui.toggle.navbar");
            ui.close();
        }
    });
    if let Some(cmd) = start {
        crate::menus::activate(app, cmd);
    }
}

/// The drawing window's close button at the drawing area's top-right corner.
pub(super) fn drawing_close(app: &mut CadApp, ui: &mut egui::Ui, rect: Rect) {
    let t = Tokens::get();
    let br = Rect::from_min_size(pos2(rect.right() - 20.0, rect.top() + 4.0), vec2(14.0, 14.0));
    let resp = ui.interact(br, ui.scope_id().with("drawing_close"), egui::Sense::click());
    let p = ui.painter_at(rect);
    if resp.hovered() {
        p.rect_filled(br, 2.0, t.control_hover);
    }
    crate::icons::paint(&p, br.shrink(2.0), crate::icons::Icon::Close, false);
    crate::icons::describe_control(ui, &resp, "Close Drawing", None, false);
    if resp.on_hover_text(crate::tl!("Close Drawing")).clicked() {
        app.request_close(Some(app.session.active));
    }
}

pub(super) fn draw_viewcube(app: &mut CadApp, ui: &mut egui::Ui, rect: Rect) {
    let l = crate::viewcube::Layout::new(rect);
    let (c, ring) = (l.center, l.ring);
    let p = ui.painter_at(rect);
    crate::viewcube::show(app, ui, &l);
    // WCS pill: the label and a down chevron, drawn as lines rather than a "⌄" glyph that a UI
    // font may lack (it showed as a box, #1541).
    let pill = Rect::from_center_size(c + vec2(0.0, ring + 26.0), vec2(56.0, 16.0));
    p.rect_filled(pill, 8.0, Color32::from_rgb(0x48, 0x50, 0x5c));
    let ink = Tokens::DARK.canvas_ink;
    let label = p.layout_no_wrap(crate::tl!("WCS").to_string(), crate::theme::small(), ink);
    let (gap, chevron) = (4.0, 6.0);
    let left = pill.center().x - (label.size().x + gap + chevron) / 2.0;
    let mid = pill.center().y;
    p.galley(pos2(left, mid - label.size().y / 2.0), label.clone(), ink);
    let x = left + label.size().x + gap;
    p.line(vec![pos2(x, mid - 1.5), pos2(x + chevron / 2.0, mid + 1.5), pos2(x + chevron, mid - 1.5)], Stroke::new(1.2, ink));
}
