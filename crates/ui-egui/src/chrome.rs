//! Window chrome: title and tool bar, file tabs, status bar and the Start page.

use egui::{Rect, Sense, Stroke, pos2, vec2};
use serde_json::json;

use crate::CadApp;
use crate::icons::{self, Icon};
use crate::theme::Tokens;

pub mod annoscale;

/// The Tool Sets & Palettes workspace's Tool Bar: (icon, command, tooltip, shortcut).
const TOOLBAR: &[&[(Icon, &str, &str, Option<&str>)]] = &[
    &[
        (Icon::New, "new", "New Drawing", Some("Cmd+N")),
        (Icon::Open, "open", "Open", Some("Cmd+O")),
        (Icon::Save, "qsave", "Save", Some("Cmd+S")),
        (Icon::SaveAs, "saveas", "Save As", None),
    ],
    &[(Icon::Undo, "u", "Undo", Some("Cmd+Z")), (Icon::Redo, "redo", "Redo", Some("Cmd+Shift+Z"))],
    &[
        (Icon::Plot, "plot", "Print", None),
        (Icon::Publish, "publish", "Batch Publish", None),
        (Icon::PageSetup, "pagesetup", "Page Setup Manager", None),
        (Icon::Preview, "preview", "Plot Preview", None),
    ],
    &[
        (Icon::Import, "import", "Import", None),
        (Icon::Export, "export", "Export", None),
        (Icon::Attach, "attach", "Attach", None),
        (Icon::Share, "share", "Share", None),
    ],
    &[(Icon::ZoomWindow, "zoom.window", "Zoom Window", None), (Icon::Pan, "pan", "Pan", None), (Icon::Orbit, "3dorbit", "Orbit", None)],
    &[
        (Icon::ZoomExtents, "zoom.extents", "Zoom Extents", None),
        (Icon::Properties, "ui.toggle.palettes", "Properties", None),
        (Icon::Layers, "ui.dialog.layers", "Layer Properties Manager", None),
        (Icon::Blocks, "ui.dialog.blocks", "Blocks", None),
    ],
    &[(Icon::Measure, "dist", "Measure Distance", None), (Icon::List, "list", "List", None), (Icon::Area, "area", "Area", None)],
    &[(Icon::Help, "ui.dialog.about", "Help", None)],
];

/// The quick access bar's commands, in groups: (icon, command, tooltip, shortcut).
const QUICK_ACCESS: &[&[(Icon, &str, &str, Option<&str>)]] = &[
    &[
        (Icon::New, "new", "New", Some("Cmd+N")),
        (Icon::Open, "open", "Open", Some("Cmd+O")),
        (Icon::Save, "qsave", "Save", Some("Cmd+S")),
        (Icon::SaveAs, "saveas", "Save As", None),
        (Icon::Plot, "plot", "Plot", Some("Cmd+P")),
        (Icon::Undo, "u", "Undo", Some("Cmd+Z")),
        (Icon::Redo, "redo", "Redo", Some("Cmd+Y")),
    ],
    &[(Icon::Share, "share", "Share", None)],
];

/// Height of the quick access bar where the system draws the window's own title bar.
const QUICK_ACCESS_BAR: f32 = 32.0;

/// The workspaces offered by the quick access bar: (name, ribbon, Tool Sets and palettes).
pub const WORKSPACES: &[(&str, bool)] = &[("Drafting & Annotation", true), ("Tool Sets & Palettes", false)];

/// The current workspace's name: the ribbon layout, or the Tool Sets and palettes one.
pub fn workspace_name(app: &CadApp) -> &'static str {
    let ribbon = app.ui.show_ribbon && !app.ui.show_toolsets;
    WORKSPACES.iter().find(|(_, r)| *r == ribbon).map_or("Drafting & Annotation", |(n, _)| n)
}

/// Switch to a workspace from [`WORKSPACES`].
pub fn set_workspace(app: &mut CadApp, ribbon: bool) {
    if ribbon {
        app.ui.use_ribbon_workspace();
    } else {
        app.ui.show_ribbon = false;
        app.ui.show_toolsets = true;
        app.ui.show_palettes = true;
        app.ui.in_window_menu = !cfg!(target_os = "macos");
    }
}

/// Height of the macOS integrated title row (#161). AppKit draws the traffic lights itself,
/// centred in the standard 28 pt title-bar band at the top of the window; moving them would take
/// AppKit calls (`unsafe`, which the workspace forbids). So the title row is that band, and what
/// it holds is centred on the same line as the lights.
const MAC_TITLE_BAR: f32 = 28.0;
/// The traffic lights as AppKit lays them out: button diameter, centre-to-centre pitch and the
/// first button's left edge (8–10 pt depending on the macOS version; the larger value keeps the
/// gap after them from shrinking).
const MAC_LIGHT: f32 = 12.0;
const MAC_LIGHT_PITCH: f32 = 20.0;
const MAC_LIGHTS_LEFT: f32 = 10.0;
/// Where title-row content may start to the right of the traffic lights: the end of the three
/// lights plus the same gap the lights keep from the top of the window.
const MAC_CONTENT_LEFT: f32 = MAC_LIGHTS_LEFT + 2.0 * MAC_LIGHT_PITCH + MAC_LIGHT + (MAC_TITLE_BAR - MAC_LIGHT) / 2.0;

/// Left edge of the window title in a title row starting at `left`, `width` wide: centred, but
/// never under the traffic lights (a long drawing name in a narrow window slides right instead).
fn title_left(left: f32, width: f32, text_w: f32) -> f32 {
    (left + (width - text_w) / 2.0).max(left + MAC_CONTENT_LEFT)
}

/// The application button's red tile, with the app icon's set square drawn in white.
fn app_button_mark(p: &egui::Painter, rect: Rect, hot: bool) {
    let red = if hot { egui::Color32::from_rgb(0xe0, 0x30, 0x38) } else { egui::Color32::from_rgb(0xc8, 0x22, 0x2c) };
    let tile = Rect::from_min_size(rect.min, vec2(rect.height(), rect.height()));
    p.rect_filled(tile, 3.0, red);
    let s = tile.height() / 24.0;
    let at = |x: f32, y: f32| pos2(tile.left() + x * s, tile.top() + y * s);
    let white = Stroke::new(1.6, egui::Color32::WHITE);
    p.line(vec![at(5.0, 5.0), at(5.0, 19.0), at(19.0, 19.0), at(5.0, 5.0)], white);
    p.line(vec![at(8.5, 11.5), at(8.5, 15.5), at(12.5, 15.5), at(8.5, 11.5)], Stroke::new(1.2, egui::Color32::WHITE));
    let k = pos2(tile.right() + 7.0, rect.center().y);
    p.add(egui::Shape::convex_polygon(vec![k + vec2(-3.0, -1.5), k + vec2(3.0, -1.5), k + vec2(0.0, 2.0)], egui::Color32::WHITE, Stroke::NONE));
}

/// The application menu: the file commands on the left, recent documents on the right, Options
/// and Exit underneath, and every menu of the menu bar under All Menus.
fn application_menu(app: &mut CadApp, ui: &mut egui::Ui, rect: Rect) {
    let resp = ui.interact(rect, ui.scope_id().with("app_menu"), Sense::click());
    app_button_mark(ui.painter(), rect, resp.hovered());
    icons::describe_control(ui, &resp, "Application Menu", None, false);
    let mut clicked: Option<String> = None;
    let mut open: Option<String> = None;
    let tree = crate::menus::tree(app);
    let recent = app.ui.recent_files.clone();
    egui::Popup::menu(&resp).show(|ui| {
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_min_width(200.0);
                for (icon, cmd, label) in [
                    (Icon::New, "new", "New"),
                    (Icon::Open, "ui.open", "Open"),
                    (Icon::Save, "qsave", "Save"),
                    (Icon::SaveAs, "ui.saveas", "Save As"),
                    (Icon::Import, "import", "Import"),
                    (Icon::Export, "export", "Export"),
                    (Icon::Publish, "publish", "Publish"),
                    (Icon::Plot, "plot", "Print"),
                    (Icon::Close, "close", "Close"),
                ] {
                    let r = ui.add(egui::Button::new(format!("        {}", crate::i18n::t(label))).min_size(vec2(200.0, 30.0)));
                    icons::paint(ui.painter(), Rect::from_center_size(pos2(r.rect.left() + 16.0, r.rect.center().y), vec2(20.0, 20.0)), icon, false);
                    icons::describe_control(ui, &r, label, None, false);
                    if r.clicked() {
                        clicked = Some(cmd.to_string());
                        ui.close();
                    }
                }
                ui.menu_button(format!("        {}", crate::tl!("Drawing Utilities")), |ui| {
                    for (cmd, label) in [("audit", "Audit"), ("purge", "Purge"), ("units", "Units"), ("ui.dialog.properties", "Drawing Properties")] {
                        if (crate::menus::is_ui_command(cmd) || autocraft_engine::find_command(cmd).is_some())
                            && ui.button(crate::i18n::t(label)).clicked()
                        {
                            clicked = Some(cmd.to_string());
                            ui.close();
                        }
                    }
                });
                let all = ui.menu_button(format!("        {}", crate::tl!("All Menus")), |ui| {
                    for (name, entries) in &tree {
                        ui.menu_button(crate::i18n::menu(name), |ui| {
                            let mut c = None;
                            for e in entries {
                                crate::menus::entry_ui(ui, e, &mut c);
                            }
                            if c.is_some() {
                                clicked = c;
                            }
                        });
                    }
                });
                icons::describe_control(ui, &all.response, "All Menus", None, false);
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.set_min_width(260.0);
                ui.label(egui::RichText::new(crate::tl!("Recent Documents")).strong());
                ui.separator();
                if recent.is_empty() {
                    ui.weak(crate::tl!("No recent documents"));
                }
                for path in &recent {
                    let r = ui.add(egui::Button::new(crate::menus::file_name(path)).frame(false)).on_hover_text(path);
                    if r.clicked() {
                        open = Some(path.clone());
                        ui.close();
                    }
                }
            });
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.add_space(220.0);
            if ui.button(crate::tl!("Options")).clicked() {
                clicked = Some("options".into());
                ui.close();
            }
            if ui.button(crate::tl!("Exit AutoCraft")).clicked() {
                clicked = Some("ui.quit".into());
                ui.close();
            }
        });
    });
    resp.on_hover_text(crate::tl!("Application Menu"));
    if let Some(path) = open {
        app.open_path(std::path::Path::new(&path));
    }
    if let Some(id) = clicked {
        crate::menus::activate(app, &id);
    }
}

/// The workspace list in the quick access bar; returns its right edge.
fn workspace_switch(app: &mut CadApp, ui: &mut egui::Ui, x: f32, cy: f32) -> f32 {
    let t = Tokens::get();
    let name = workspace_name(app);
    let galley = ui.painter().layout_no_wrap(crate::i18n::t(name).to_string(), crate::theme::small(), t.text);
    let rect = Rect::from_min_size(pos2(x, cy - 10.0), vec2(galley.size().x + 40.0, 20.0));
    let resp = ui.interact(rect, ui.scope_id().with("workspace_switch"), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, 3.0, t.control_hover.gamma_multiply(0.6));
    }
    icons::paint(ui.painter(), Rect::from_center_size(pos2(rect.left() + 10.0, cy), vec2(14.0, 14.0)), Icon::Gear, false);
    ui.painter().galley(pos2(rect.left() + 21.0, cy - galley.size().y / 2.0), galley, t.text);
    let k = pos2(rect.right() - 8.0, cy);
    ui.painter().add(egui::Shape::convex_polygon(vec![k + vec2(-3.0, -1.5), k + vec2(3.0, -1.5), k + vec2(0.0, 2.0)], t.text_dim, Stroke::NONE));
    icons::describe_control(ui, &resp, "Workspace", None, false);
    workspace_menu(app, &resp);
    rect.right()
}

/// The workspaces under `resp`; a choice switches to it.
pub fn workspace_menu(app: &mut CadApp, resp: &egui::Response) {
    let name = workspace_name(app);
    let mut pick = None;
    egui::Popup::menu(resp).show(|ui| {
        for (n, ribbon) in WORKSPACES {
            let r = ui.add(egui::Button::selectable(*n == name, crate::i18n::t(n)));
            crate::control::record_widget(ui, &r, n, "button", Some(*n == name));
            if r.clicked() {
                pick = Some(*ribbon);
                ui.close();
            }
        }
    });
    if let Some(ribbon) = pick {
        set_workspace(app, ribbon);
    }
}

/// The search box at the right of the title bar: Enter runs what was typed as a command, the
/// way the command line would.
fn search_box(app: &mut CadApp, ui: &mut egui::Ui, rect: Rect) {
    let t = Tokens::get();
    let id = ui.scope_id().with("title_search");
    let mut text = ui.data(|d| d.get_temp::<String>(id)).unwrap_or_default();
    ui.painter().rect_filled(rect, 2.0, t.field);
    ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    icons::paint(ui.painter(), Rect::from_center_size(pos2(rect.right() - 11.0, rect.center().y), vec2(13.0, 13.0)), Icon::Search, false);
    let inner = Rect::from_min_max(rect.min + vec2(6.0, 1.0), pos2(rect.right() - 22.0, rect.bottom() - 1.0));
    let mut run = None;
    ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
        let edit = egui::TextEdit::singleline(&mut text)
            .id(id.with("edit"))
            .hint_text(crate::tl!("Type a keyword or phrase"))
            .font(crate::theme::small())
            .frame(egui::Frame::NONE)
            .desired_width(inner.width());
        let r = ui.add(edit);
        crate::control::record_widget(ui, &r, "Search", "textbox", None);
        if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !text.trim().is_empty() {
            run = Some(std::mem::take(&mut text));
        }
    });
    ui.data_mut(|d| d.insert_temp(id, text));
    if let Some(cmd) = run {
        let _ = app.cmdline(cmd.trim());
    }
}

/// Width of one drawn window button (minimize, maximize, close) on Windows.
const CAPTION_BUTTON_W: f32 = 46.0;

/// The window's minimize, maximize/restore and close buttons at the right of `r`, for windows the
/// app draws its own title bar on. Returns the left edge of the first button.
pub fn caption_buttons(ui: &mut egui::Ui, r: Rect) -> f32 {
    let t = Tokens::get();
    let maximized = ui.input(|i| i.viewport().maximized).unwrap_or(false);
    let mut x = r.right() - 3.0 * CAPTION_BUTTON_W;
    let left = x;
    for (k, label) in ["Minimize", if maximized { "Restore Down" } else { "Maximize" }, "Close"].into_iter().enumerate() {
        let br = Rect::from_min_size(pos2(x, r.top()), vec2(CAPTION_BUTTON_W, r.height()));
        let resp = ui.interact(br, ui.scope_id().with(("caption", k)), Sense::click());
        let close = k == 2;
        if resp.hovered() {
            ui.painter().rect_filled(br, 0.0, if close { egui::Color32::from_rgb(0xc4, 0x2b, 0x1c) } else { t.control_hover.gamma_multiply(0.7) });
        }
        let c = br.center();
        let st = Stroke::new(1.0, t.text);
        match k {
            0 => {
                ui.painter().hline((c.x - 5.0)..=(c.x + 5.0), c.y, st);
            }
            1 if maximized => {
                ui.painter().rect_stroke(Rect::from_center_size(c + vec2(-1.0, 1.0), vec2(8.0, 8.0)), 0.0, st, egui::StrokeKind::Middle);
                ui.painter().line(vec![c + vec2(-3.0, -3.0), c + vec2(-3.0, -5.0), c + vec2(5.0, -5.0), c + vec2(5.0, 3.0), c + vec2(3.0, 3.0)], st);
            }
            1 => {
                ui.painter().rect_stroke(Rect::from_center_size(c, vec2(10.0, 10.0)), 0.0, st, egui::StrokeKind::Middle);
            }
            _ => {
                ui.painter().line_segment([c + vec2(-5.0, -5.0), c + vec2(5.0, 5.0)], st);
                ui.painter().line_segment([c + vec2(5.0, -5.0), c + vec2(-5.0, 5.0)], st);
            }
        }
        icons::describe_control(ui, &resp, label, None, false);
        if resp.clicked() {
            let cmd = match k {
                0 => egui::ViewportCommand::Minimized(true),
                1 => egui::ViewportCommand::Maximized(!maximized),
                _ => egui::ViewportCommand::Close,
            };
            ui.ctx().send_viewport_cmd(cmd);
        }
        x += CAPTION_BUTTON_W;
    }
    left
}

/// The window title: the app and the drawing.
fn window_title(app: &CadApp) -> String {
    match app.session.state() {
        Ok(st) if !app.ui.start_tab => format!("AutoCraft  {}{}", st.title, if st.title.contains('.') { "" } else { ".dwg" }),
        _ => format!("AutoCraft  {}", crate::tl!("Start")),
    }
}

/// Height of the title bar the app draws (with the quick access bar on it).
fn title_bar_height(app: &CadApp) -> f32 {
    if app.integrated_titlebar {
        MAC_TITLE_BAR
    } else if app.caption_buttons || app.ui.show_toolbar {
        QUICK_ACCESS_BAR
    } else {
        0.0
    }
}

/// The title bar along the top of the window, as AutoCAD lays it out: the application button and
/// the quick access commands, the workspace list and Share, the drawing's title in the middle,
/// then the search box, Help and (where the app draws the title bar) the window buttons.
pub fn title_and_toolbar(app: &mut CadApp, ui: &mut egui::Ui) {
    if !app.ui.show_ribbon {
        classic_title_and_toolbar(app, ui);
        return;
    }
    let t = Tokens::get();
    let bar_h = title_bar_height(app);
    if bar_h <= 0.0 {
        return;
    }
    egui::Panel::top("cc_title_toolbar").exact_size(bar_h).frame(egui::Frame::NONE.fill(t.title_bar)).show(ui, |ui| {
        let r = ui.max_rect();
        let draws_title = app.integrated_titlebar || app.caption_buttons;
        if draws_title {
            // The empty bar moves the window; it goes first so later widgets take their own clicks.
            craft_window::title_bar::empty_area(ui, r);
        }
        let cy = r.center().y;
        let mut right = r.right() - 6.0;
        if app.caption_buttons {
            right = caption_buttons(ui, r) - 4.0;
        }
        let mut x = r.left() + if app.integrated_titlebar { MAC_CONTENT_LEFT } else { 6.0 };
        let mut qat_right = x;
        let size = 22.0;
        let mut clicked = None;
        if app.ui.show_toolbar {
            application_menu(app, ui, Rect::from_min_size(pos2(x, cy - 10.0), vec2(34.0, 20.0)));
            x += 42.0;
            let mac = ui.ctx().os().is_mac();
            for (g, group) in QUICK_ACCESS.iter().enumerate() {
                if g > 0 {
                    ui.painter().vline(x + 3.0, (cy - 7.0)..=(cy + 7.0), Stroke::new(1.0, t.border));
                    x += 8.0;
                }
                for (icon, cmd, label, shortcut) in group.iter() {
                    if x + size > right - 8.0 {
                        break;
                    }
                    let br = Rect::from_min_size(pos2(x, cy - size / 2.0), vec2(size, size));
                    let resp = ui.interact(br, ui.scope_id().with(("tb", *cmd)), Sense::click());
                    if resp.hovered() {
                        ui.painter().rect_filled(br, 3.0, t.control_hover.gamma_multiply(0.6));
                    }
                    icons::paint(ui.painter(), br.shrink(3.5), *icon, false);
                    let tip = match shortcut {
                        Some(sc) => format!("{} ({})", crate::i18n::t(label), crate::menus::shortcut_label(sc, mac)),
                        None => crate::i18n::t(label).to_string(),
                    };
                    icons::describe_control(ui, &resp, &tip, None, false);
                    if resp.on_hover_text(tip).clicked() {
                        clicked = Some(*cmd);
                    }
                    x += size + 2.0;
                }
            }
            x += 10.0;
            if x + 200.0 < right - 40.0 {
                x = workspace_switch(app, ui, x, cy) + 8.0;
            }
            qat_right = x;
            // Help, then the search box, from the right.
            let hr = Rect::from_min_size(pos2(right - size, cy - size / 2.0), vec2(size, size));
            if hr.left() > qat_right {
                let resp = ui.interact(hr, ui.scope_id().with("tb_help"), Sense::click());
                if resp.hovered() {
                    ui.painter().rect_filled(hr, 3.0, t.control_hover.gamma_multiply(0.6));
                }
                icons::paint(ui.painter(), hr.shrink(3.5), Icon::Help, false);
                icons::describe_control(ui, &resp, "Help", None, false);
                if resp.on_hover_text(crate::tl!("Help")).clicked() {
                    clicked = Some("ui.dialog.commands");
                }
                right = hr.left() - 8.0;
            }
            let sr = Rect::from_min_max(pos2(right - 210.0, cy - 10.0), pos2(right, cy + 10.0));
            if sr.left() > qat_right + 320.0 {
                search_box(app, ui, sr);
                right = sr.left() - 8.0;
            }
        }
        if draws_title || app.ui.show_toolbar {
            let galley = ui.painter().layout_no_wrap(window_title(app), egui::FontId::proportional(12.5), t.text_dim);
            let tx = title_left(r.left(), r.width(), galley.size().x).max(r.left() + (r.width() - galley.size().x) / 2.0);
            if tx > qat_right + 8.0 && tx + galley.size().x < right - 8.0 {
                ui.painter().galley(pos2(tx, r.top() + (bar_h - galley.size().y) / 2.0), galley, t.text_dim);
            }
        }
        if let Some(c) = clicked {
            app.start(c);
        }
    });
}

/// The Tool Sets & Palettes workspace's title row (with the integrated macOS title bar) and
/// full Tool Bar.
fn classic_title_and_toolbar(app: &mut CadApp, ui: &mut egui::Ui) {
    let t = Tokens::get();
    let title_h = if app.integrated_titlebar {
        MAC_TITLE_BAR
    } else if app.caption_buttons {
        30.0
    } else {
        0.0
    };
    let tb_h = if app.ui.show_toolbar { 34.0 } else { 0.0 };
    egui::Panel::top("cc_title_toolbar").exact_size(title_h + tb_h).frame(egui::Frame::NONE.fill(t.chrome)).show(ui, |ui| {
        let r = ui.max_rect();
        if title_h > 0.0 {
            // The empty row moves the window; it goes first so later widgets take their own clicks.
            craft_window::title_bar::empty_area(ui, Rect::from_min_size(r.min, vec2(r.width(), title_h)));
            if app.caption_buttons {
                caption_buttons(ui, Rect::from_min_size(r.min, vec2(r.width(), title_h)));
            }
            let title = match app.session.state() {
                Ok(st) if !app.ui.start_tab => format!("AutoCraft      {}{}", st.title, if st.title.contains('.') { "" } else { ".dwg" }),
                _ => format!("AutoCraft      {}", crate::tl!("Start")),
            };
            // Centred on the traffic lights' centre line (the row's middle): with the system font
            // the capitals' middle is the text box's middle, so no optical nudge.
            let galley = ui.painter().layout_no_wrap(title, egui::FontId::proportional(13.0), t.text_dim);
            let x = title_left(r.left(), r.width(), galley.size().x);
            ui.painter().galley(pos2(x, r.top() + (title_h - galley.size().y) / 2.0), galley, t.text_dim);
        }
        if tb_h > 0.0 {
            let row = Rect::from_min_size(pos2(r.left(), r.top() + title_h), vec2(r.width(), tb_h));
            let mut x = r.left() + if app.integrated_titlebar { 120.0 } else { 12.0 };
            let size = 26.0;
            let mut clicked = None;
            let mac = ui.ctx().os().is_mac();
            let widths: Vec<_> = TOOLBAR.iter().map(|group| group.len() as f32 * (size + 6.0) - 6.0).collect();
            let available = (row.right() - 12.0 - x).max(0.0);
            let Some(fit) = craft_ui::layout::fit_toolbar(&widths, available, 24.0, 30.0) else {
                // Very small windows still expose the complete command row by scrolling.
                ui.scope_builder(egui::UiBuilder::new().max_rect(row.shrink2(vec2(4.0, 0.0))), |ui| {
                    egui::ScrollArea::horizontal().id_salt("toolbar_small").show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for group in TOOLBAR {
                                for (icon, cmd, tip, _) in *group {
                                    if icons::button(ui, *icon, size, tip, false).clicked() {
                                        clicked = Some(*cmd);
                                    }
                                }
                                ui.add_space(18.0);
                            }
                        });
                    });
                });
                if let Some(command) = clicked {
                    app.start(command);
                }
                return;
            };
            for (index, _) in &fit.shown {
                let Some(group) = TOOLBAR.get(*index) else { continue };
                for (icon, cmd, label, shortcut) in group.iter() {
                    let br = Rect::from_min_size(pos2(x, row.center().y - size / 2.0), vec2(size, size));
                    let resp = ui.interact(br, ui.scope_id().with(("tb", *cmd)), Sense::click());
                    if resp.hovered() {
                        ui.painter().rect_filled(br, 3.0, t.control_hover.gamma_multiply(0.6));
                    }
                    icons::paint(ui.painter(), br.shrink(4.0), *icon, false);
                    let tip = match shortcut {
                        Some(sc) => format!("{} ({})", crate::i18n::t(label), crate::menus::shortcut_label(sc, mac)),
                        None => crate::i18n::t(label).to_string(),
                    };
                    icons::describe_control(ui, &resp, &tip, None, false);
                    if resp.on_hover_text(tip).clicked() {
                        clicked = Some(*cmd);
                    }
                    x += size + 6.0;
                }
                x += 18.0;
            }
            if !fit.overflow.is_empty() {
                let br = Rect::from_min_size(pos2(x, row.center().y - size / 2.0), vec2(fit.overflow_width, size));
                let response = ui.interact(br, ui.scope_id().with("toolbar_more"), Sense::click());
                if response.hovered() {
                    ui.painter().rect_filled(br, 3.0, t.control_hover.gamma_multiply(0.6));
                }
                for offset in [-5.0, 0.0, 5.0] {
                    ui.painter().circle_filled(br.center() + vec2(offset, 0.0), 1.5, t.text);
                }
                icons::describe_control(ui, &response, "More toolbar commands", None, false);
                egui::Popup::menu(&response).show(|ui| {
                    for (position, index) in fit.overflow.iter().enumerate() {
                        let Some(group) = TOOLBAR.get(*index) else { continue };
                        if position > 0 {
                            ui.separator();
                        }
                        for (_, command, label, _) in *group {
                            let response = ui.button(crate::i18n::t(label));
                            crate::control::record_widget(ui, &response, label, "button", None);
                            if response.clicked() {
                                clicked = Some(*command);
                                ui.close();
                            }
                        }
                    }
                });
                response.on_hover_text(crate::tl!("More toolbar commands"));
            }
            if let Some(c) = clicked {
                app.start(c);
            }
        }
    });
}

/// File tabs: switcher, Start, open drawings, then "+".
pub fn file_tabs(app: &mut CadApp, ui: &mut egui::Ui) {
    let t = Tokens::get();
    egui::Panel::top("cc_file_tabs").exact_size(26.0).frame(egui::Frame::NONE.fill(t.chrome_dark)).show(ui, |ui| {
        let r = ui.max_rect();
        let mut x = r.left() + 4.0;
        // The file tab menu: every open drawing, as AutoCAD's lists them.
        let ib = Rect::from_min_size(pos2(x, r.top() + 4.0), vec2(18.0, 18.0));
        let iresp = ui.interact(ib, ui.scope_id().with("file_tab_menu"), Sense::click());
        if iresp.hovered() {
            ui.painter().rect_filled(ib, 3.0, t.control_hover.gamma_multiply(0.6));
        }
        icons::paint(ui.painter(), ib.shrink(2.0), if app.ui.show_ribbon { Icon::Menu } else { Icon::Switcher }, false);
        icons::describe_control(ui, &iresp, "Drawings", None, false);
        let mut menu_pick = None;
        let names: Vec<String> = app.session.docs.iter().map(|d| d.title.clone()).collect();
        let active_doc = app.session.active;
        egui::Popup::menu(&iresp).show(|ui| {
            for (i, n) in names.iter().enumerate() {
                if ui.add(egui::Button::selectable(i == active_doc && !app.ui.start_tab, n)).clicked() {
                    menu_pick = Some(i);
                    ui.close();
                }
            }
        });
        if let Some(i) = menu_pick {
            let _ = app.run("document.switch", json!({ "index": i }));
            app.ui.start_tab = false;
        }
        x += 24.0;
        // `label` is the drawing's name; `dirty` adds the unsaved-changes mark, kept visible when a
        // long name is cut short.
        let mut tab =
            |ui: &mut egui::Ui, label: &str, dirty: bool, active: bool, closable: bool, w: f32, id: egui::Id| -> (bool, bool, egui::Response) {
                let tr = Rect::from_min_size(pos2(x, r.top()), vec2(w, r.height()));
                let resp = ui.interact(tr, id, Sense::click());
                // The close button takes the tab's hover, so `resp.hovered()` would make the tab and
                // its X flicker while the pointer is on the X.
                let hovered = ui.rect_contains_pointer(tr);
                ui.painter().rect_filled(
                    tr,
                    0.0,
                    if active {
                        t.tab_active
                    } else if hovered {
                        t.chrome
                    } else {
                        t.chrome_dark
                    },
                );
                // The name ends in "…" when it doesn't fit before the close button (#513).
                let color = if active { t.text } else { t.text_dim };
                let mark = if dirty { "*" } else { "" };
                let text_w = (w - 8.0 - if closable { 24.0 } else { 8.0 }).max(0.0);
                let mark_w = ui.painter().layout_no_wrap(mark.to_string(), crate::theme::body(), color).size().x;
                let (galley, cut) = elided(ui.painter(), label, crate::theme::body(), color, text_w - mark_w);
                let pos = pos2(tr.left() + 8.0, tr.center().y - galley.size().y / 2.0);
                let name_w = galley.size().x;
                ui.painter().galley(pos, galley, color);
                if dirty {
                    ui.painter().text(pos2(pos.x + name_w, tr.center().y), egui::Align2::LEFT_CENTER, mark, crate::theme::body(), color);
                }
                let resp = if cut { resp.on_hover_text(format!("{label}{mark}")) } else { resp };
                let mut close = false;
                if closable && (active || hovered) {
                    let cr = Rect::from_center_size(pos2(tr.right() - 11.0, tr.center().y), vec2(12.0, 12.0));
                    let cresp = ui.interact(cr, id.with("x"), Sense::click());
                    icons::paint(ui.painter(), cr, Icon::Close, false);
                    icons::describe_control(ui, &cresp, &format!("Close {label}"), None, false);
                    close = cresp.clicked();
                }
                icons::describe_control(ui, &resp, label, Some(active), true);
                ui.painter().vline(tr.right(), tr.y_range(), Stroke::new(1.0, t.border));
                x += w;
                (resp.clicked(), close, resp)
            };
        let (c, _, _) =
            tab(ui, crate::tl!("Start"), false, app.ui.start_tab || app.session.docs.is_empty(), false, 110.0, ui.scope_id().with("tab_start"));
        if c {
            app.ui.start_tab = true;
        }
        let mut switch_to = None;
        let mut close = None;
        let titles: Vec<(usize, String, bool, bool)> =
            app.session.docs.iter().enumerate().map(|(i, d)| (i, d.title.clone(), d.is_dirty(), i == app.session.active)).collect();
        for (i, title, dirty, active) in titles {
            let (c, x, resp) = tab(ui, &title, dirty, active && !app.ui.start_tab, true, 170.0, ui.scope_id().with(("tab", i)));
            crate::shortcut_menus::attach(app, &resp, |app| crate::shortcut_menus::drawing_tab_entries(app, i));
            if c {
                switch_to = Some(i);
            }
            if x {
                close = Some(i);
            }
        }
        // "+" after the last drawing opens a new one.
        let plus = Rect::from_min_size(pos2(x + 4.0, r.top() + 4.0), vec2(18.0, 18.0));
        let presp = ui.interact(plus, ui.scope_id().with("tab+"), Sense::click());
        if presp.hovered() {
            ui.painter().rect_filled(plus, 3.0, t.control_hover.gamma_multiply(0.6));
        }
        icons::paint(ui.painter(), plus.shrink(2.0), Icon::Plus, false);
        icons::describe_control(ui, &presp, "New Drawing", None, false);
        if presp.on_hover_text(crate::tl!("New Drawing")).clicked() {
            app.start("new");
            app.ui.start_tab = false;
        }
        if let Some(i) = switch_to {
            let _ = app.run("document.switch", json!({ "index": i }));
            app.ui.start_tab = false;
        }
        if let Some(i) = close {
            app.request_close(Some(i));
        }
    });
}

/// `text` laid out on one line no wider than `max_width`, ending in "…" when it had to be cut;
/// and whether it was.
fn elided(p: &egui::Painter, text: &str, font: egui::FontId, color: egui::Color32, max_width: f32) -> (std::sync::Arc<egui::Galley>, bool) {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_string(), font, color);
    job.wrap = egui::text::TextWrapping { max_width: max_width.max(0.0), max_rows: 1, break_anywhere: true, overflow_character: Some('…') };
    let galley = p.layout_job(job);
    let cut = galley.elided;
    (galley, cut)
}

/// Widest layout tab name before it is cut with "…".
const LAYOUT_TAB_TEXT: f32 = 150.0;
/// Room the layout tabs leave for the coordinates and messages.
const STATUS_READOUT_MIN: f32 = 150.0;

/// Which of the tabs with these `widths` to show in `room`: all of them when they fit, else a run
/// that holds the `active` one (shown even when it alone is too wide), starting as early as it can.
fn visible_tabs(widths: &[f32], active: Option<usize>, room: f32) -> std::ops::Range<usize> {
    let fits = |range: std::ops::Range<usize>| widths.get(range).is_some_and(|w| w.iter().sum::<f32>() <= room);
    let mut end = 0;
    while end < widths.len() && fits(0..end + 1) {
        end += 1;
    }
    let Some(active) = active.filter(|a| *a >= end && *a < widths.len()) else { return 0..end };
    let mut start = active;
    while start > 0 && fits(start - 1..active + 1) {
        start -= 1;
    }
    start..active + 1
}

/// Status bar buttons hidden until turned on in the status bar's Customization menu, as in
/// AutoCAD's default status bar.
pub const DEFAULT_STATUS_HIDDEN: &[&str] = &["coordinates", "dynmode", "lwdisplay", "transparencydisplay", "selectioncycling", "qpmode"];

/// What a status bar button's arrow (or right-click) opens.
#[derive(Clone, Copy, PartialEq, Eq)]
enum StatusMenu {
    Snap,
    Polar,
    Iso,
    Osnap,
    Workspace,
    Isolate,
}

/// A status bar button: icon, on, command, tooltip, the menu beside it.
type StatusButton = (Icon, bool, &'static str, &'static str, Option<StatusMenu>);

/// The status bar's drafting buttons in AutoCAD's order (each can be hidden in Customization).
fn status_buttons(app: &CadApp) -> Vec<StatusButton> {
    use autocraft_engine::snap::mode;
    let s = &app.session.settings;
    let anno_all = app.session.doc().is_ok_and(|d| d.anno_all_visible());
    vec![
        (Icon::Grid, s.gridmode, "grid", "Grid (F7)", None),
        (Icon::Snap, s.snapmode, "snap", "Snap Mode (F9)", Some(StatusMenu::Snap)),
        (Icon::DynInput, s.dynmode, "dynmode", "Dynamic Input (F12)", None),
        (Icon::Ortho, s.orthomode, "ortho", "Ortho Mode (F8)", None),
        (Icon::Polar, s.polarmode, "polar", "Polar Tracking (F10)", Some(StatusMenu::Polar)),
        (Icon::Isodraft, s.isodraft, "isodraft", "Isometric Drafting", Some(StatusMenu::Iso)),
        (Icon::OTrack, s.otrack, "otrack", "Object Snap Tracking (F11)", None),
        (Icon::Osnap, s.osmode & mode::OFF == 0 && s.osmode != 0, "osnap", "Object Snap (F3)", Some(StatusMenu::Osnap)),
        (Icon::Lineweight, s.lwdisplay, "lwdisplay", "Show/Hide Lineweight", None),
        (Icon::Transparency, s.transparency_display, "transparencydisplay", "Show/Hide Transparency", None),
        (Icon::SelectObjects, s.selection_cycling, "selectioncycling", "Selection Cycling", None),
        (Icon::Annotation, anno_all, "annovisible.toggle", "Annotation Visibility", None),
    ]
}

/// The buttons after the annotation scale.
fn status_tail(app: &CadApp) -> Vec<StatusButton> {
    vec![
        (Icon::Workspace, false, "ui.noop", "Workspace Switching", Some(StatusMenu::Workspace)),
        (Icon::QuickProps, app.session.settings.qpmode, "qpmode", "Quick Properties", None),
        (Icon::LayerIso, false, "ui.noop", "Isolate Objects", Some(StatusMenu::Isolate)),
        (Icon::ZoomFit, app.ui.clean_screen.is_some(), "ui.cleanscreen", "Clean Screen", None),
    ]
}

/// The menu of a status bar button.
fn status_menu(app: &mut CadApp, ui: &mut egui::Ui, menu: StatusMenu) {
    use autocraft_engine::snap::mode;
    let settings = |ui: &mut egui::Ui, label: &str, app: &mut CadApp| {
        ui.separator();
        if ui.button(crate::i18n::t(label)).clicked() {
            app.start("ui.dialog.dsettings");
            ui.close();
        }
    };
    match menu {
        StatusMenu::Snap => {
            let on = app.session.settings.snapmode;
            if ui.add(egui::Button::selectable(on, crate::tl!("Grid Snap"))).clicked() {
                crate::menus::run_transparent(app, "snap");
                ui.close();
            }
            settings(ui, "Snap Settings...", app);
        }
        StatusMenu::Polar => {
            let cur = app.session.settings.polarang;
            for a in [90.0, 45.0, 30.0, 22.5, 18.0, 15.0, 10.0, 5.0] {
                if ui.add(egui::Button::selectable((cur - a).abs() < 1e-6, format!("{a}, {}, {}, …", a * 2.0, a * 3.0))).clicked() {
                    let _ = app.run("dsettings", json!({ "polarang": a }));
                    if !app.session.settings.polarmode {
                        crate::menus::run_transparent(app, "polar");
                    }
                    ui.close();
                }
            }
            settings(ui, "Tracking Settings...", app);
        }
        StatusMenu::Iso => {
            let (on, plane) = (app.session.settings.isodraft, app.session.settings.isoplane);
            for (label, p, pl) in [
                ("Isoplane Left", "left", autocraft_engine::snap::iso::IsoPlane::Left),
                ("Isoplane Top", "top", autocraft_engine::snap::iso::IsoPlane::Top),
                ("Isoplane Right", "right", autocraft_engine::snap::iso::IsoPlane::Right),
            ] {
                if ui.add(egui::Button::selectable(on && plane == pl, crate::i18n::t(label))).clicked() {
                    let _ = app.run("isodraft", json!({ "on": true, "plane": p }));
                    ui.close();
                }
            }
            if ui.add(egui::Button::selectable(!on, crate::tl!("Orthographic"))).clicked() {
                let _ = app.run("isodraft", json!({ "on": false }));
                ui.close();
            }
        }
        StatusMenu::Osnap => {
            let bits = app.session.settings.osmode;
            for (bit, name) in mode::ALL {
                let mut on = bits & bit != 0;
                let r = ui.checkbox(&mut on, crate::i18n::t(name));
                crate::control::record_widget(ui, &r, name, "checkbox", Some(on));
                if r.changed() {
                    // A choice turns object snap on, as AutoCAD's list does.
                    let next = if on { bits | bit } else { bits & !bit } & !mode::OFF;
                    let _ = app.run("osnap", json!({ "osmode": next }));
                }
            }
            settings(ui, "Object Snap Settings...", app);
        }
        StatusMenu::Workspace => {
            let name = workspace_name(app);
            for (n, ribbon) in WORKSPACES {
                let r = ui.add(egui::Button::selectable(*n == name, crate::i18n::t(n)));
                crate::control::record_widget(ui, &r, n, "button", Some(*n == name));
                if r.clicked() {
                    set_workspace(app, *ribbon);
                    ui.close();
                }
            }
        }
        StatusMenu::Isolate => {
            for (cmd, label) in [("isolateobjects", "Isolate Objects"), ("hideobjects", "Hide Objects"), ("unisolateobjects", "End Object Isolation")]
            {
                if ui.button(crate::i18n::t(label)).clicked() {
                    crate::menus::activate(app, cmd);
                    ui.close();
                }
            }
        }
    }
}

/// Width of a status bar button: the icon, and the arrow when it has a menu.
fn status_button_w(b: &StatusButton) -> f32 {
    20.0 + if b.4.is_some() { 9.0 } else { 0.0 }
}

/// Draw `buttons` from `x`, `rect`'s row; returns the right edge.
fn draw_status_toggles(app: &mut CadApp, ui: &mut egui::Ui, painter: &egui::Painter, buttons: &[StatusButton], rect: Rect, mut x: f32) -> f32 {
    let t = Tokens::get();
    let size = 20.0;
    let mut toggle_cmd = None;
    for &(icon, on, cmd, tip, menu) in buttons {
        let br = Rect::from_min_size(pos2(x, rect.center().y - size / 2.0), vec2(size, size));
        let resp = ui.interact(br, ui.scope_id().with(("tg", tip)), Sense::click());
        if on {
            painter.rect_filled(br, 3.0, t.toggle_on.gamma_multiply(0.35));
        } else if resp.hovered() {
            painter.rect_filled(br, 3.0, t.control_hover.gamma_multiply(0.5));
        }
        icons::paint(painter, br.shrink(2.5), icon, false);
        if on {
            painter.hline(br.x_range().shrink(4.0), br.bottom() - 1.0, Stroke::new(1.5, t.toggle_on));
        }
        icons::describe_control(ui, &resp, tip, if cmd == "ui.noop" { None } else { Some(on) }, false);
        let resp = resp.on_hover_text(crate::i18n::t(tip));
        x += size;
        if let Some(m) = menu {
            let ar = Rect::from_min_size(pos2(x, br.top()), vec2(9.0, size));
            let aresp = ui.interact(ar, ui.scope_id().with(("tg_menu", tip)), Sense::click());
            if aresp.hovered() {
                painter.rect_filled(ar, 2.0, t.control_hover.gamma_multiply(0.5));
            }
            let c = ar.center();
            painter.add(egui::Shape::convex_polygon(vec![c + vec2(-3.0, -1.5), c + vec2(3.0, -1.5), c + vec2(0.0, 2.0)], t.text_dim, Stroke::NONE));
            icons::describe_control(ui, &aresp, &format!("{tip} menu"), None, false);
            // Buttons without a toggle of their own open the menu from the icon too.
            let opener = if cmd == "ui.noop" { &resp } else { &aresp };
            egui::Popup::menu(opener).show(|ui| status_menu(app, ui, m));
            if cmd != "ui.noop" {
                egui::Popup::context_menu(&resp).show(|ui| status_menu(app, ui, m));
            }
            x += 9.0;
        }
        if cmd != "ui.noop" && resp.clicked() {
            toggle_cmd = Some(cmd);
        }
        x += 3.0;
    }
    // The toggles don't cancel the active command (#1108); the dialog buttons start theirs.
    match toggle_cmd {
        Some(c) if c.starts_with("ui.") => app.start(c),
        Some(c) => crate::menus::run_transparent(app, c),
        None => {}
    }
    x
}

/// The status bar's Customization menu: which buttons show.
fn customization_menu(app: &mut CadApp, ui: &mut egui::Ui) {
    let mut entries: Vec<(&'static str, &'static str)> = vec![("coordinates", "Coordinates")];
    entries.extend(status_buttons(app).iter().chain(status_tail(app).iter()).filter(|b| b.2 != "ui.noop").map(|b| (b.2, b.3)));
    for (cmd, label) in entries {
        let mut shown = !app.ui.status_hidden.iter().any(|h| h == cmd);
        if ui.checkbox(&mut shown, crate::i18n::t(label)).changed() {
            app.ui.status_hidden.retain(|h| h != cmd);
            if !shown {
                app.ui.status_hidden.push(cmd.to_string());
            }
        }
    }
}

/// Status bar: the layout list, Model and the layout tabs and "+" on the left; on the right,
/// MODEL, the drafting buttons, the annotation scale, workspace, isolate, Clean Screen and the
/// Customization menu, in AutoCAD's order.
pub fn status_bar(app: &mut CadApp, ui: &mut egui::Ui) {
    let t = Tokens::get();
    egui::Panel::bottom("cc_status").exact_size(26.0).frame(egui::Frame::NONE.fill(t.chrome)).show(ui, |ui| {
        let r = ui.max_rect();
        let p = ui.painter().clone();
        let mut x = r.left() + 8.0;
        // Clear the status message on time even when no input arrives (the app only repaints on input).
        if let Some((_, at)) = &app.status
            && crate::now_ms() - at < 5000.0
        {
            let left = std::time::Duration::try_from_secs_f64((5000.0 - (crate::now_ms() - at)) / 1000.0).unwrap_or_default();
            ui.ctx().request_repaint_after(left);
        }
        let br = Rect::from_min_size(pos2(x, r.top() + 4.0), vec2(18.0, 18.0));
        let list_button = ui.interact(br, ui.scope_id().with(("sb", "Layout list")), Sense::click());
        if list_button.hovered() {
            p.rect_filled(br, 3.0, t.control_hover.gamma_multiply(0.5));
        }
        icons::paint(&p, br.shrink(2.0), Icon::Menu, false);
        icons::describe_control(ui, &list_button, "Layout list", None, false);
        let list_button = list_button.on_hover_text(crate::tl!("Layout list"));
        x += 26.0;
        // Right side, laid out from the right edge: Customization, then the rest leftwards.
        let hidden = app.ui.status_hidden.clone();
        let shown = |b: &&StatusButton| !hidden.iter().any(|h| h == b.2);
        let head: Vec<StatusButton> = status_buttons(app).iter().filter(shown).copied().collect();
        let tail: Vec<StatusButton> = status_tail(app).iter().filter(shown).copied().collect();
        let cust = Rect::from_min_size(pos2(r.right() - 26.0, r.center().y - 10.0), vec2(20.0, 20.0));
        let cresp = ui.interact(cust, ui.scope_id().with("status_customize"), Sense::click());
        if cresp.hovered() {
            p.rect_filled(cust, 3.0, t.control_hover.gamma_multiply(0.5));
        }
        icons::paint(&p, cust.shrink(3.0), Icon::Menu, false);
        icons::describe_control(ui, &cresp, "Customization", None, false);
        egui::Popup::menu(&cresp).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside).show(|ui| customization_menu(app, ui));
        let tail_w: f32 = tail.iter().map(|b| status_button_w(b) + 3.0).sum();
        let tail_x = cust.left() - 6.0 - tail_w;
        draw_status_toggles(app, ui, &p, &tail, r, tail_x);
        // The annotation scale sits between the drafting buttons and the workspace button.
        let scale_right = annoscale::status(app, ui, tail_x - 4.0, r.center().y);
        let head_w: f32 = head.iter().map(|b| status_button_w(b) + 3.0).sum();
        let head_x = scale_right - 6.0 - head_w;
        draw_status_toggles(app, ui, &p, &head, r, head_x);
        // MODEL / PAPER: in a layout it switches between paper space and the viewport's model space.
        let paper = matches!(app.session.layout_space(), autocraft_doc::Space::Paper(_));
        let model_vp = app.session.state().is_ok_and(|st| st.mspace.is_some());
        let label = if !paper || model_vp { "MODEL" } else { "PAPER" };
        let lg = p.layout_no_wrap(crate::i18n::t(label).to_string(), crate::theme::small(), t.text);
        let mr = Rect::from_min_max(pos2(head_x - 10.0 - lg.size().x - 12.0, r.center().y - 9.0), pos2(head_x - 8.0, r.center().y + 9.0));
        let mresp = ui.interact(mr, ui.scope_id().with("mspace_toggle"), Sense::click());
        p.rect_filled(mr, 3.0, if mresp.hovered() { t.control_hover } else { t.chrome_dark });
        p.galley(pos2(mr.center().x - lg.size().x / 2.0, mr.center().y - lg.size().y / 2.0), lg, t.text);
        icons::describe_control(ui, &mresp, label, None, false);
        let mut coord_right = mr.left() - 10.0;
        let mut switch = None;
        if mresp.on_hover_text(crate::tl!("Switch between model space in a viewport and paper space")).clicked() {
            if paper {
                let _ = app.run(if model_vp { "pspace" } else { "mspace" }, json!({}));
                app.canvas.list = None;
            } else if let Ok(st) = app.session.state() {
                // In model space, MODEL opens the first layout.
                let mut ls: Vec<_> = st.doc.layouts.iter().collect();
                ls.sort_by_key(|l| l.tab_order);
                switch = ls.first().map(|l| l.name.clone());
            }
        }
        if paper {
            coord_right = crate::vpscale::status(app, ui, coord_right + 4.0, r.center().y) - 10.0;
        }
        // Coordinates (when turned on in Customization).
        let coord = (!hidden.iter().any(|h| h == "coordinates")).then_some(app.canvas.cursor).flatten().map(|c| {
            let (lu, lp) = app.session.doc().map(|d| (d.header.i64("LUNITS", 2), d.header.i64("LUPREC", 4))).unwrap_or((2, 4));
            format!(
                "{}, {}, {}",
                autocraft_engine::units::format_distance(c.x, lu, lp),
                autocraft_engine::units::format_distance(c.y, lu, lp),
                autocraft_engine::units::format_distance(0.0, lu, lp)
            )
        });
        // Layout tabs: Model, the layouts that fit before the right-hand controls (the current one
        // always), then "+"; the layout list reaches the rest.
        let mut tabs: Vec<String> = vec!["Model".into()];
        let cur = app.session.state().map(|s| match &s.space {
            autocraft_doc::Space::Model => "Model".to_string(),
            autocraft_doc::Space::Paper(n) => n.clone(),
        });
        if let Ok(st) = app.session.state() {
            let mut ls: Vec<_> = st.doc.layouts.iter().collect();
            ls.sort_by_key(|l| l.tab_order);
            tabs.extend(ls.iter().map(|l| l.name.clone()));
        }
        let shown_name = |i: usize, name: &str| if i == 0 { crate::tl!("Model").to_string() } else { name.to_string() };
        let limit = (coord_right - if coord.is_some() { STATUS_READOUT_MIN } else { 40.0 }).max(x);
        let labels: Vec<_> =
            tabs.iter().enumerate().map(|(i, n)| elided(&p, &shown_name(i, n), crate::theme::body(), t.text, LAYOUT_TAB_TEXT)).collect();
        let widths: Vec<f32> = labels.iter().map(|(g, _)| g.size().x + 26.0 + 2.0).collect();
        let active_tab = tabs.iter().position(|n| cur.as_deref().is_ok_and(|c| c == n));
        let layouts = widths.get(1..).unwrap_or_default();
        let room = limit - x - widths.first().copied().unwrap_or(0.0) - 44.0;
        let active_layout = active_tab.and_then(|a| a.checked_sub(1));
        let mut shown_tabs = visible_tabs(layouts, active_layout, room);
        let cut = shown_tabs.len() < layouts.len();
        if cut {
            shown_tabs = visible_tabs(layouts, active_layout, room - 22.0);
        }
        let clip = Rect::from_min_max(r.min, pos2(limit.max(r.left()), r.bottom()));
        let tp = p.with_clip_rect(clip);
        for (i, (name, (g, elide))) in tabs.iter().zip(labels).enumerate() {
            if i > 0 && !shown_tabs.contains(&(i - 1)) {
                continue;
            }
            let w = g.size().x + 26.0;
            let tr = Rect::from_min_size(pos2(x, r.top() + 1.0), vec2(w, r.height() - 2.0)).intersect(clip);
            let active = active_tab == Some(i);
            let resp = ui.interact(tr, ui.scope_id().with(("lt", i)), Sense::click());
            if active {
                tp.rect_filled(tr, 2.0, t.tab_active);
            } else if resp.hovered() {
                tp.rect_filled(tr, 2.0, t.control.gamma_multiply(0.5));
            }
            tp.galley(pos2(tr.left() + 13.0, tr.center().y - g.size().y / 2.0), g, if active { t.text } else { t.text_dim });
            icons::describe_control(ui, &resp, name, Some(active), true);
            let resp = if elide { resp.on_hover_text(shown_name(i, name)) } else { resp };
            if resp.clicked() {
                switch = Some(name.clone());
            }
            crate::shortcut_menus::attach(app, &resp, |app| crate::shortcut_menus::layout_tab_entries(app, name));
            x += w + 2.0;
        }
        // The layouts that didn't fit, behind a button after the last tab shown.
        let more = cut.then(|| {
            let mr = Rect::from_min_size(pos2(x + 2.0, r.top() + 5.0), vec2(16.0, 16.0));
            let resp = ui.interact(mr, ui.scope_id().with("layout_more"), Sense::click());
            if resp.hovered() {
                p.rect_filled(mr, 3.0, t.control_hover.gamma_multiply(0.5));
            }
            icons::paint(&p, mr.shrink(2.0), Icon::ChevronDown, false);
            icons::describe_control(ui, &resp, "More layouts", None, false);
            x += 22.0;
            resp.on_hover_text(crate::tl!("Layout list"))
        });
        // "+" after the tabs adds a layout.
        let pr = Rect::from_min_size(pos2(x + 2.0, r.top() + 5.0), vec2(16.0, 16.0));
        let presp = ui.interact(pr, ui.scope_id().with("layout_plus"), Sense::click());
        if presp.hovered() {
            p.rect_filled(pr, 3.0, t.control_hover.gamma_multiply(0.5));
        }
        icons::paint(&p, pr.shrink(2.0), Icon::Plus, false);
        icons::describe_control(ui, &presp, "New layout", None, false);
        if presp.on_hover_text(crate::tl!("New layout")).clicked()
            && let Ok(v) = app.run("layout.new", json!({}))
            && let Some(n) = v.get("name").and_then(serde_json::Value::as_str)
        {
            switch = Some(n.to_string());
        }
        x += 24.0;
        for resp in std::iter::once(&list_button).chain(&more) {
            egui::Popup::from_toggle_button_response(resp).width(200.0).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside).show(|ui| {
                egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                    for (i, name) in tabs.iter().enumerate() {
                        if ui.add(egui::Button::selectable(active_tab == Some(i), shown_name(i, name)).truncate()).clicked() {
                            switch = Some(name.clone());
                            ui.close();
                        }
                    }
                });
            });
        }
        if let Some(n) = switch {
            let _ = app.run("layout.set", json!({ "name": n }));
        }
        // A transient message, or the coordinates, in the room left before MODEL.
        let message = app.status.as_ref().filter(|(_, at)| crate::now_ms() - at < 5000.0).map(|(message, _)| message);
        let readout = message.cloned().or(coord);
        if let Some(text) = readout {
            let left = x + 8.0;
            let width = (coord_right - left).max(0.0);
            if width > 4.0 {
                let color = if message.is_some() { t.warn } else { t.text };
                let mut job = egui::text::LayoutJob::simple_singleline(text.clone(), crate::theme::body(), color);
                job.wrap.max_width = width;
                job.wrap.max_rows = 1;
                job.wrap.break_anywhere = true;
                let galley = p.layout_job(job);
                let rect = Rect::from_min_max(pos2(left, r.top()), pos2(coord_right, r.bottom()));
                let text_x = if message.is_some() { left } else { coord_right - galley.size().x };
                p.with_clip_rect(rect).galley(pos2(text_x, r.center().y - galley.size().y / 2.0), galley, color);
                ui.interact(rect, ui.scope_id().with("status_readout"), Sense::hover()).on_hover_text(text);
            }
        }
    });
}

/// The Start page (no drawing open, or the Start tab).
pub fn start_page(app: &mut CadApp, ui: &mut egui::Ui) {
    let t = Tokens::get();
    let r = ui.max_rect();
    let p = ui.painter().clone();
    let left = Rect::from_min_size(r.min, vec2(300.0, r.height()));
    p.rect_filled(left, 0.0, t.chrome_dark);
    p.rect_filled(Rect::from_min_max(pos2(left.right(), r.top()), r.max), 0.0, t.chrome);
    p.text(pos2(left.left() + 34.0, left.top() + 70.0), egui::Align2::LEFT_CENTER, crate::tl!("AutoCraft"), egui::FontId::proportional(28.0), t.text);
    p.text(
        pos2(left.left() + 34.0, left.top() + 98.0),
        egui::Align2::LEFT_CENTER,
        crate::tf!("Version {version}", version = env!("CARGO_PKG_VERSION")),
        crate::theme::small(),
        t.text_faint,
    );
    let mut y = left.top() + 140.0;
    let mut action = None;
    for (label, cmd) in [("Open...", "open"), ("New", "new"), ("New (metric)", "new.metric"), ("Open sample drawing", "sample")] {
        let br = Rect::from_min_size(pos2(left.left() + 34.0, y), vec2(220.0, 34.0));
        let resp = ui.interact(br, ui.scope_id().with(("start", cmd)), Sense::click());
        p.rect_stroke(br, 2.0, Stroke::new(1.0, if resp.hovered() { t.text } else { t.text_dim }), egui::StrokeKind::Inside);
        p.text(pos2(br.left() + 16.0, br.center().y), egui::Align2::LEFT_CENTER, crate::i18n::t(label), crate::theme::body(), t.text);
        if resp.clicked() {
            action = Some(cmd);
        }
        y += 44.0;
    }
    let main = Rect::from_min_max(pos2(left.right() + 36.0, r.top() + 40.0), r.max);
    p.text(main.min, egui::Align2::LEFT_TOP, crate::tl!("Recent"), egui::FontId::proportional(22.0), t.text);
    let mut recent = None;
    let mut y = main.top() + 44.0;
    for (i, path) in app.ui.recent_files.iter().enumerate() {
        let row = Rect::from_min_size(pos2(main.left(), y), vec2((r.right() - main.left() - 36.0).clamp(0.0, 640.0), 42.0));
        let resp = ui.interact(row, ui.scope_id().with(("recent", i)), Sense::click());
        if resp.hovered() {
            p.rect_filled(row, 2.0, t.control_hover.gamma_multiply(0.5));
        }
        let dir = std::path::Path::new(path).parent().map(|d| d.display().to_string()).unwrap_or_default();
        let clip = p.with_clip_rect(row.shrink(2.0));
        clip.text(pos2(row.left() + 10.0, row.top() + 13.0), egui::Align2::LEFT_CENTER, crate::menus::file_name(path), crate::theme::body(), t.text);
        clip.text(pos2(row.left() + 10.0, row.top() + 30.0), egui::Align2::LEFT_CENTER, dir, crate::theme::small(), t.text_faint);
        if resp.on_hover_text(path).clicked() {
            recent = Some(path.clone());
        }
        y += 46.0;
    }
    if let Some(path) = recent {
        app.open_path(std::path::Path::new(&path));
    } else if app.ui.recent_files.is_empty() {
        p.text(
            pos2(main.center().x, main.center().y),
            egui::Align2::CENTER_CENTER,
            crate::tl!("There is nothing here yet."),
            egui::FontId::proportional(18.0),
            t.text,
        );
        p.text(
            pos2(main.center().x, main.center().y + 26.0),
            egui::Align2::CENTER_CENTER,
            crate::tl!("To get started, create or open a drawing."),
            crate::theme::body(),
            t.text_dim,
        );
    }
    match action {
        Some("open") => app.start("ui.open"),
        Some("new") => {
            app.start("new");
            app.ui.start_tab = false;
        }
        Some("new.metric") => {
            let _ = app.run("new", json!({ "metric": true }));
            app.ui.start_tab = false;
        }
        Some("sample") => {
            app.start("ui.sample");
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_title_row_centres_and_clears_the_traffic_lights() {
        // The lights are centred in the row, and content keeps the same gap after them.
        let margin = (MAC_TITLE_BAR - MAC_LIGHT) / 2.0;
        let lights_end = MAC_LIGHTS_LEFT + 2.0 * MAC_LIGHT_PITCH + MAC_LIGHT;
        assert_eq!(MAC_CONTENT_LEFT - lights_end, margin);
        // A short title is centred; one too wide for that starts after the lights.
        assert_eq!(title_left(100.0, 1000.0, 200.0), 500.0);
        assert_eq!(title_left(100.0, 900.0, 800.0), 100.0 + MAC_CONTENT_LEFT);
    }

    #[test]
    fn layout_tabs_that_do_not_fit_keep_the_current_one() {
        let w = [50.0; 6];
        // All fit, with or without a current tab.
        assert_eq!(visible_tabs(&w, None, 300.0), 0..6);
        assert_eq!(visible_tabs(&w, Some(5), 300.0), 0..6);
        // Too many: the first ones, unless the current one is further on.
        assert_eq!(visible_tabs(&w, None, 170.0), 0..3);
        assert_eq!(visible_tabs(&w, Some(1), 170.0), 0..3);
        assert_eq!(visible_tabs(&w, Some(4), 170.0), 2..5);
        // No room at all: nothing but the current tab; hostile numbers don't panic.
        assert_eq!(visible_tabs(&w, None, -10.0), 0..0);
        assert_eq!(visible_tabs(&w, Some(3), 0.0), 3..4);
        assert_eq!(visible_tabs(&w, Some(9), f32::NAN), 0..0);
        assert_eq!(visible_tabs(&[], Some(0), 100.0), 0..0);
    }
}
