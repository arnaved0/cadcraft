//! The Drafting & Annotation workspace: quick access bar, ribbon, file tabs, drawing area and
//! status bar. Ribbon buttons start their commands, panels that do not fit collapse to a menu,
//! the application menu reaches the menu bar's commands, and the workspace switches back to Tool
//! Sets and the palettes.

use cadcraft_engine::Session;
use cadcraft_ui_egui::{CadApp, Services};
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

fn harness(width: f32) -> Harness<'static, CadApp> {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", serde_json::json!({})).unwrap();
    app.ui.start_tab = false;
    app.ui.use_ribbon_workspace();
    let mut h = Harness::builder().with_size(vec2(width, 800.0)).build_ui_state(
        |ui, app: &mut CadApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(4);
    h
}

fn running(h: &Harness<'_, CadApp>) -> Option<String> {
    h.state().session.running.as_ref().map(|r| r.id.clone())
}

#[test]
fn ribbon_workspace_hides_tool_sets_palettes_and_menu_bar() {
    let h = harness(1600.0);
    assert!(h.state().ui.show_ribbon);
    assert!(h.query_by_role_and_label(egui::accesskit::Role::Tab, "Tool Sets").is_none());
    assert!(h.query_by_role_and_label(egui::accesskit::Role::Tab, "Properties").is_none());
    for tab in ["Home", "Insert", "Annotate", "Parametric", "View", "Manage", "Output"] {
        assert!(h.query_by_role_and_label(egui::accesskit::Role::Tab, tab).is_some(), "{tab}");
    }
    h.get_by_label("Application Menu");
    h.get_by_label("Model");
}

#[test]
fn ribbon_buttons_start_their_commands() {
    let mut h = harness(1600.0);
    h.get_by_label("Line").click();
    h.run_steps(2);
    assert_eq!(running(&h).as_deref(), Some("line"));
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    h.get_by_label("Move").click();
    h.run_steps(2);
    assert_eq!(running(&h).as_deref(), Some("move"));
}

#[test]
fn ribbon_tabs_switch_panels() {
    let mut h = harness(1600.0);
    h.get_by_role_and_label(egui::accesskit::Role::Tab, "Annotate").click();
    h.run_steps(2);
    assert_eq!(h.state().ui.ribbon_tab, "Annotate");
    h.get_by_label("Multileader").click();
    h.run_steps(2);
    assert_eq!(running(&h).as_deref(), Some("mleader"));
}

#[test]
fn panels_that_do_not_fit_collapse_into_a_menu() {
    let mut h = harness(900.0);
    // Clipboard is the last Home panel; at this width it is one button that opens a menu.
    h.get_by_label("Clipboard").click();
    h.run_steps(2);
    h.get_by_label("Paste").click();
    h.run_steps(2);
    assert!(h.query_by_label("Cut").is_none(), "the menu closes after a choice");
}

#[test]
fn minimized_ribbon_keeps_its_tabs() {
    let mut h = harness(1600.0);
    h.get_by_label("Minimize Ribbon").click();
    h.run_steps(2);
    assert!(h.state().ui.ribbon_minimized);
    assert!(h.query_by_label("Line").is_none());
    h.get_by_role_and_label(egui::accesskit::Role::Tab, "Home").click();
    h.run_steps(2);
    assert!(!h.state().ui.ribbon_minimized);
    h.get_by_label("Line");
}

#[test]
fn application_menu_holds_the_menus() {
    let mut h = harness(1600.0);
    h.get_by_label("Application Menu").click();
    h.run_steps(2);
    h.get_by_label("File");
    h.get_by_label("Draw");
}

#[test]
fn workspace_switch_returns_to_tool_sets_and_palettes() {
    let mut h = harness(1600.0);
    h.get_by_label("Workspace Switching").click();
    h.run_steps(3);
    assert!(!h.state().ui.show_ribbon);
    assert!(h.state().ui.show_toolsets && h.state().ui.show_palettes);
    h.get_by_role_and_label(egui::accesskit::Role::Tab, "Tool Sets");
    h.get_by_label("Workspace Switching").click();
    h.run_steps(3);
    assert!(h.state().ui.show_ribbon && !h.state().ui.show_toolsets);
}
