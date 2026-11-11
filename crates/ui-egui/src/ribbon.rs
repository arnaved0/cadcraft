//! The ribbon: tabs of command panels across the top of the window, under the quick access bar.
//!
//! Each tab holds panels; each panel holds large buttons (icon over label), columns of small
//! labelled buttons, grids of small icon buttons, or one of the current-property lists. A panel
//! that does not fit collapses to a single button that opens the panel's commands in a menu.
//! Every button reaches its command through [`crate::menus::activate`], like the menus do.

use autocraft_color::Color;
use egui::{Align2, Color32, Rect, Sense, Stroke, pos2, vec2};
use serde_json::json;

use crate::CadApp;
use crate::icons::{self, Icon};
use crate::theme::Tokens;

/// A command button: icon, command id and label.
pub type Tool = (Icon, &'static str, &'static str);

/// What a ribbon panel holds, left to right.
#[derive(Clone, Copy)]
pub enum Item {
    /// A large button: the icon over its label.
    Large(Tool),
    /// A column of up to three small buttons with labels.
    Column(&'static [Tool]),
    /// Small icon-only buttons, three rows high, as many columns as they need.
    Icons(&'static [Tool]),
    /// The current layer list.
    LayerList,
    /// The colour, linetype and lineweight of new objects.
    PropertyLists,
}

/// A ribbon panel: its title (shown under it), contents, and the slide-out under its title.
pub struct Panel {
    pub name: &'static str,
    pub items: &'static [Item],
    /// The tools of the panel's slide-out (the arrow beside its title); empty for none.
    pub more: &'static [Tool],
}

/// A ribbon tab.
pub struct Tab {
    pub name: &'static str,
    pub panels: &'static [Panel],
}

use Item::{Column, Icons, Large, LayerList, PropertyLists};

const fn panel(name: &'static str, items: &'static [Item], more: &'static [Tool]) -> Panel {
    Panel { name, items, more }
}

/// Split buttons: a button whose command is listed here shows an arrow that opens these
/// variants; the variant last chosen becomes the button's command (its first entry until then).
pub const SPLITS: &[(&str, &[Tool])] = &[
    (
        "circle",
        &[
            (Icon::Circle, "circle", "Center, Radius"),
            (Icon::Circle, "circle.cd", "Center, Diameter"),
            (Icon::Circle, "circle.2p", "2-Point"),
            (Icon::Circle, "circle.3p", "3-Point"),
            (Icon::Circle, "circle.ttr", "Tan, Tan, Radius"),
            (Icon::Circle, "circle.ttt", "Tan, Tan, Tan"),
        ],
    ),
    (
        "arc",
        &[
            (Icon::Arc, "arc", "3-Point"),
            (Icon::Arc, "arc.sce", "Start, Center, End"),
            (Icon::Arc, "arc.sca", "Start, Center, Angle"),
            (Icon::Arc, "arc.scl", "Start, Center, Length"),
            (Icon::Arc, "arc.sea", "Start, End, Angle"),
            (Icon::Arc, "arc.sed", "Start, End, Direction"),
            (Icon::Arc, "arc.ser", "Start, End, Radius"),
            (Icon::Arc, "arc.cse", "Center, Start, End"),
            (Icon::Arc, "arc.csa", "Center, Start, Angle"),
            (Icon::Arc, "arc.csl", "Center, Start, Length"),
            (Icon::Arc, "arc.continue", "Continue"),
        ],
    ),
    ("rectang", &[(Icon::Rectangle, "rectang", "Rectangle"), (Icon::Polygon, "polygon", "Polygon")]),
    ("hatch", &[(Icon::Hatch, "hatch", "Hatch"), (Icon::Gradient, "gradient", "Gradient"), (Icon::Boundary, "boundary", "Boundary")]),
    (
        "ellipse",
        &[(Icon::Ellipse, "ellipse", "Center"), (Icon::Ellipse, "ellipse.axis", "Axis, End"), (Icon::Ellipse, "ellipse.arc", "Elliptical Arc")],
    ),
    ("spline", &[(Icon::Spline, "spline", "Spline Fit"), (Icon::Spline, "spline.cv", "Spline CV")]),
    ("trim", &[(Icon::Trim, "trim", "Trim"), (Icon::Extend, "extend", "Extend")]),
    ("fillet", &[(Icon::Fillet, "fillet", "Fillet"), (Icon::Chamfer, "chamfer", "Chamfer"), (Icon::Spline, "blend", "Blend Curves")]),
    (
        "arrayrect",
        &[
            (Icon::ArrayRect, "arrayrect", "Rectangular Array"),
            (Icon::Spline, "arraypath", "Path Array"),
            (Icon::ArrayPolar, "arraypolar", "Polar Array"),
        ],
    ),
    ("mtext", &[(Icon::MText, "mtext", "Multiline Text"), (Icon::Text, "text", "Single Line")]),
    (
        "dim",
        &[
            (Icon::DimLinear, "dim", "Dimension"),
            (Icon::DimLinear, "dimlinear", "Linear"),
            (Icon::DimAligned, "dimaligned", "Aligned"),
            (Icon::DimAngular, "dimangular", "Angular"),
            (Icon::DimArc, "dimarc", "Arc Length"),
            (Icon::DimRadius, "dimradius", "Radius"),
            (Icon::DimDiameter, "dimdiameter", "Diameter"),
            (Icon::DimOrdinate, "dimordinate", "Ordinate"),
        ],
    ),
    (
        "dimlinear",
        &[
            (Icon::DimLinear, "dimlinear", "Linear"),
            (Icon::DimAligned, "dimaligned", "Aligned"),
            (Icon::DimAngular, "dimangular", "Angular"),
            (Icon::DimArc, "dimarc", "Arc Length"),
            (Icon::DimRadius, "dimradius", "Radius"),
            (Icon::DimDiameter, "dimdiameter", "Diameter"),
            (Icon::DimOrdinate, "dimordinate", "Ordinate"),
        ],
    ),
    ("dimcontinue", &[(Icon::DimContinue, "dimcontinue", "Continue"), (Icon::DimBaseline, "dimbaseline", "Baseline")]),
    (
        "mleader",
        &[
            (Icon::MLeader, "mleader", "Multileader"),
            (Icon::AddLeader, "mleaderedit", "Add Leader"),
            (Icon::RemoveLeader, "mleaderedit.remove", "Remove Leader"),
        ],
    ),
    ("insert", &[(Icon::Insert, "insert", "Insert"), (Icon::Blocks, "ui.dialog.blocks", "Blocks Palette")]),
    ("attedit", &[(Icon::AttDef, "attedit", "Single"), (Icon::AttDef, "eattext", "Extract Data")]),
    ("measuregeom", &[(Icon::Measure, "measuregeom", "Measure"), (Icon::Distance, "dist", "Distance"), (Icon::Area, "area", "Area")]),
    (
        "pasteclip",
        &[
            (Icon::Paste, "pasteclip", "Paste"),
            (Icon::Paste, "pasteblock", "Paste as Block"),
            (Icon::Paste, "pasteorig", "Paste to Original Coordinates"),
        ],
    ),
    ("copyclip", &[(Icon::Copy, "copyclip", "Copy Clip"), (Icon::Copy, "copybase", "Copy with Base Point")]),
    (
        "zoom.extents",
        &[
            (Icon::ZoomExtents, "zoom.extents", "Zoom Extents"),
            (Icon::ZoomWindow, "zoom.window", "Zoom Window"),
            (Icon::ZoomFit, "zoom.previous", "Zoom Previous"),
            (Icon::ZoomIn, "zoom.realtime", "Zoom Realtime"),
            (Icon::ZoomFit, "zoom.all", "Zoom All"),
            (Icon::ZoomIn, "zoom.in", "Zoom In"),
            (Icon::ZoomOut, "zoom.out", "Zoom Out"),
        ],
    ),
    (
        "vports",
        &[
            (Icon::Grid, "vports.1", "Single"),
            (Icon::Grid, "vports.2", "Two: Vertical"),
            (Icon::Grid, "vports.3", "Three: Right"),
            (Icon::Grid, "vports.4", "Four: Equal"),
            (Icon::Grid, "vports", "Viewport Configuration..."),
        ],
    ),
    ("isolateobjects", &[(Icon::LayerIso, "isolateobjects", "Isolate Objects"), (Icon::LayerOff, "hideobjects", "Hide Objects")]),
];

/// The ribbon's tabs and panels, grouped as in AutoCAD's Drafting & Annotation workspace.
pub const TABS: &[Tab] = &[
    Tab {
        name: "Home",
        panels: &[
            panel(
                "Draw",
                &[
                    Large((Icon::Line, "line", "Line")),
                    Large((Icon::Polyline, "pline", "Polyline")),
                    Large((Icon::Circle, "circle", "Circle")),
                    Large((Icon::Arc, "arc", "Arc")),
                    Icons(&[
                        (Icon::Rectangle, "rectang", "Rectangle"),
                        (Icon::Hatch, "hatch", "Hatch"),
                        (Icon::Spline, "spline", "Spline"),
                        (Icon::Ellipse, "ellipse", "Ellipse"),
                        (Icon::XLine, "xline", "Construction Line"),
                        (Icon::Point, "point.multiple", "Multiple Points"),
                    ]),
                ],
                &[
                    (Icon::Ray, "ray", "Ray"),
                    (Icon::Point, "divide", "Divide"),
                    (Icon::Point, "measure", "Measure"),
                    (Icon::Boundary, "boundary", "Boundary"),
                    (Icon::Region, "region", "Region"),
                    (Icon::Wipeout, "wipeout", "Wipeout"),
                    (Icon::RevCloud, "revcloud", "Revision Cloud"),
                    (Icon::Donut, "donut", "Donut"),
                ],
            ),
            panel(
                "Modify",
                &[
                    Column(&[(Icon::Move, "move", "Move"), (Icon::Copy, "copy", "Copy"), (Icon::Stretch, "stretch", "Stretch")]),
                    Column(&[(Icon::Rotate, "rotate", "Rotate"), (Icon::Mirror, "mirror", "Mirror"), (Icon::Scale, "scale", "Scale")]),
                    Column(&[(Icon::Trim, "trim", "Trim"), (Icon::Fillet, "fillet", "Fillet"), (Icon::ArrayRect, "arrayrect", "Array")]),
                    Icons(&[(Icon::Erase, "erase", "Erase"), (Icon::Explode, "explode", "Explode"), (Icon::Offset, "offset", "Offset")]),
                ],
                &[
                    (Icon::Break, "break", "Break"),
                    (Icon::Join, "join", "Join"),
                    (Icon::Lengthen, "lengthen", "Lengthen"),
                    (Icon::Polyline, "pedit", "Edit Polyline"),
                    (Icon::Spline, "splinedit", "Edit Spline"),
                    (Icon::Move, "align", "Align"),
                    (Icon::Join, "overkill", "Delete Duplicate Objects"),
                    (Icon::MoveTop, "draworder", "Draw Order"),
                ],
            ),
            panel(
                "Annotation",
                &[
                    Large((Icon::MText, "mtext", "Text")),
                    Large((Icon::DimLinear, "dim", "Dimension")),
                    Column(&[(Icon::MLeader, "mleader", "Leader"), (Icon::Table, "table", "Table")]),
                ],
                &[
                    (Icon::TextStyle, "style", "Text Style"),
                    (Icon::DimStyle, "dimstyle", "Dimension Style"),
                    (Icon::MLeader, "mleaderstyle", "Multileader Style"),
                    (Icon::Table, "tablestyle", "Table Style"),
                ],
            ),
            panel(
                "Layers",
                &[Large((Icon::LayerProps, "ui.dialog.layers", "Layer Properties")), LayerList],
                &[
                    (Icon::Bulb, "layon", "Turn All Layers On"),
                    (Icon::Sun, "laythw", "Thaw All Layers"),
                    (Icon::LayerIso, "layuniso", "Unisolate"),
                    (Icon::Unlock, "layulk", "Unlock"),
                    (Icon::LayerMatch, "laymrg", "Merge"),
                    (Icon::Erase, "laydel", "Delete"),
                    (Icon::MakeCurrent, "laycur", "Change to Current Layer"),
                ],
            ),
            panel(
                "Block",
                &[
                    Large((Icon::Insert, "insert", "Insert")),
                    Column(&[
                        (Icon::BlockCreate, "block", "Create"),
                        (Icon::BlockEdit, "bedit", "Edit"),
                        (Icon::AttDef, "attedit", "Edit Attributes"),
                    ]),
                ],
                &[
                    (Icon::AttDef, "attdef", "Define Attributes"),
                    (Icon::AttDef, "battman", "Manage Attributes"),
                    (Icon::BlockCreate, "wblock", "Write Block"),
                ],
            ),
            panel(
                "Properties",
                &[Large((Icon::MatchProp, "matchprop", "Match Properties")), PropertyLists],
                &[(Icon::List, "list", "List"), (Icon::Properties, "ui.panel.properties", "Properties Palette")],
            ),
            panel(
                "Groups",
                &[
                    Large((Icon::Group, "group", "Group")),
                    Icons(&[(Icon::Ungroup, "ungroup", "Ungroup"), (Icon::BlockEdit, "groupedit", "Group Edit")]),
                ],
                &[],
            ),
            panel(
                "Utilities",
                &[Large((Icon::Measure, "measuregeom", "Measure"))],
                &[
                    (Icon::QuickSelect, "qselect", "Quick Select"),
                    (Icon::SelectObjects, "selectall", "Select All"),
                    (Icon::Point, "id", "ID Point"),
                    (Icon::Area, "ui.toggle.quickcalc", "QuickCalc"),
                ],
            ),
            panel(
                "Clipboard",
                &[Large((Icon::Paste, "pasteclip", "Paste")), Icons(&[(Icon::Cut, "cutclip", "Cut"), (Icon::Copy, "copyclip", "Copy Clip")])],
                &[],
            ),
            panel("View", &[Large((Icon::ZoomExtents, "zoom.extents", "Zoom"))], &[(Icon::Pan, "pan", "Pan"), (Icon::Orbit, "3dorbit", "Orbit")]),
        ],
    },
    Tab {
        name: "Insert",
        panels: &[
            panel(
                "Block",
                &[
                    Large((Icon::Insert, "insert", "Insert")),
                    Column(&[
                        (Icon::AttDef, "attedit", "Edit Attribute"),
                        (Icon::AttDef, "battman", "Manage Attributes"),
                        (Icon::BlockEdit, "bedit", "Block Editor"),
                    ]),
                ],
                &[],
            ),
            panel(
                "Block Definition",
                &[
                    Large((Icon::BlockCreate, "block", "Create Block")),
                    Large((Icon::AttDef, "attdef", "Define Attributes")),
                    Column(&[
                        (Icon::AttDef, "battman", "Manage Attributes"),
                        (Icon::BlockEdit, "bedit", "Block Editor"),
                        (Icon::BlockCreate, "wblock", "Write Block"),
                    ]),
                ],
                &[],
            ),
            panel(
                "Reference",
                &[
                    Large((Icon::Attach, "attach", "Attach")),
                    Column(&[
                        (Icon::Attach, "xattach", "DWG Reference"),
                        (Icon::Attach, "xclip", "Clip"),
                        (Icon::Attach, "xref", "External References"),
                    ]),
                ],
                &[],
            ),
            panel("Import", &[Large((Icon::Import, "import", "Import"))], &[]),
            panel(
                "Linking & Extraction",
                &[
                    Large((Icon::Table, "dataextraction", "Extract Data")),
                    Column(&[(Icon::Table, "datalink", "Data Link"), (Icon::AttDef, "eattext", "Extract Attributes")]),
                ],
                &[],
            ),
            panel("Content", &[Large((Icon::Blocks, "ui.dialog.blocks", "Blocks"))], &[]),
        ],
    },
    Tab {
        name: "Annotate",
        panels: &[
            panel(
                "Text",
                &[
                    Large((Icon::MText, "mtext", "Multiline Text")),
                    Column(&[(Icon::Text, "text", "Single Line"), (Icon::Search, "find", "Find Text"), (Icon::TextStyle, "style", "Text Style")]),
                ],
                &[(Icon::MText, "txt2mtxt", "Text to Multiline Text"), (Icon::Text, "textedit", "Edit Text")],
            ),
            panel(
                "Dimensions",
                &[
                    Large((Icon::DimLinear, "dim", "Dimension")),
                    Column(&[
                        (Icon::DimLinear, "dimlinear", "Linear"),
                        (Icon::QDim, "qdim", "Quick"),
                        (Icon::DimContinue, "dimcontinue", "Continue"),
                    ]),
                    Icons(&[
                        (Icon::DimStyle, "dimstyle", "Dimension Style"),
                        (Icon::Break, "dimbreak", "Break"),
                        (Icon::DimBaseline, "dimspace", "Adjust Space"),
                        (Icon::DimLinear, "dimjogline", "Jogged Linear"),
                        (Icon::DimStyle, "tolerance", "Tolerance"),
                        (Icon::DimStyle, "dimstyle.update", "Update"),
                    ]),
                ],
                &[],
            ),
            panel("Centerlines", &[Large((Icon::Circle, "centermark", "Center Mark")), Large((Icon::Line, "centerline", "Centerline"))], &[]),
            panel(
                "Leaders",
                &[
                    Large((Icon::MLeader, "mleader", "Multileader")),
                    Column(&[
                        (Icon::AddLeader, "mleaderedit", "Add Leader"),
                        (Icon::RemoveLeader, "mleaderedit.remove", "Remove Leader"),
                        (Icon::AlignLeaders, "mleaderalign", "Align"),
                    ]),
                    Icons(&[(Icon::MLeader, "mleadercollect", "Collect"), (Icon::MLeader, "mleaderstyle", "Multileader Style")]),
                ],
                &[],
            ),
            panel(
                "Tables",
                &[
                    Large((Icon::Table, "table", "Table")),
                    Column(&[(Icon::Table, "tablestyle", "Table Style"), (Icon::Table, "datalink", "Link Data")]),
                ],
                &[],
            ),
            panel("Markup", &[Large((Icon::RevCloud, "revcloud", "Revision Cloud")), Large((Icon::Wipeout, "wipeout", "Wipeout"))], &[]),
        ],
    },
    Tab {
        name: "Parametric",
        panels: &[
            panel(
                "Geometric",
                &[
                    Large((Icon::Constraint, "autoconstrain", "Auto Constrain")),
                    Icons(&[
                        (Icon::Coincident, "gccoincident", "Coincident"),
                        (Icon::Collinear, "gccollinear", "Collinear"),
                        (Icon::Concentric, "gcconcentric", "Concentric"),
                        (Icon::Fix, "gcfix", "Fix"),
                        (Icon::Parallel, "gcparallel", "Parallel"),
                        (Icon::Perpendicular, "gcperpendicular", "Perpendicular"),
                        (Icon::Horizontal, "gchorizontal", "Horizontal"),
                        (Icon::Vertical, "gcvertical", "Vertical"),
                        (Icon::Tangent, "gctangent", "Tangent"),
                        (Icon::Smooth, "gcsmooth", "Smooth"),
                        (Icon::Symmetric, "gcsymmetric", "Symmetric"),
                        (Icon::Equal, "gcequal", "Equal"),
                    ]),
                ],
                &[],
            ),
            panel(
                "Dimensional",
                &[
                    Large((Icon::DimLinear, "dclinear", "Linear")),
                    Large((Icon::DimAligned, "dcaligned", "Aligned")),
                    Icons(&[
                        (Icon::DimRadius, "dcradius", "Radius"),
                        (Icon::DimDiameter, "dcdiameter", "Diameter"),
                        (Icon::DimAngular, "dcangular", "Angular"),
                    ]),
                ],
                &[],
            ),
            panel(
                "Manage",
                &[Large((Icon::DelConstraint, "delconstraint", "Delete Constraints")), Large((Icon::Parameters, "parameters", "Parameters Manager"))],
                &[],
            ),
        ],
    },
    Tab {
        name: "View",
        panels: &[
            panel(
                "Viewport Tools",
                &[
                    Large((Icon::Grid, "ui.toggle.ucsicon", "UCS Icon")),
                    Large((Icon::Orbit, "ui.toggle.viewcube", "View Cube")),
                    Large((Icon::Pan, "ui.toggle.navbar", "Navigation Bar")),
                ],
                &[],
            ),
            panel(
                "Model Viewports",
                &[
                    Large((Icon::Grid, "vports", "Viewport Configuration")),
                    Column(&[(Icon::Grid, "vports.named", "Named"), (Icon::ZoomFit, "view", "Named Views")]),
                ],
                &[],
            ),
            panel(
                "Palettes",
                &[
                    Large((Icon::Blocks, "ui.toggle.toolsets", "Tool Palettes")),
                    Large((Icon::Properties, "ui.panel.properties", "Properties")),
                    Column(&[
                        (Icon::Layers, "ui.panel.layers", "Layers"),
                        (Icon::Area, "ui.toggle.quickcalc", "QuickCalc"),
                        (Icon::Attach, "xref", "External References"),
                    ]),
                ],
                &[],
            ),
            panel(
                "Interface",
                &[
                    Large((Icon::Menu, "ui.toggle.filetabs", "File Tabs")),
                    Column(&[
                        (Icon::Menu, "ui.toggle.statusbar", "Status Bar"),
                        (Icon::Menu, "ui.toggle.menubar", "Menu Bar"),
                        (Icon::Menu, "ui.toggle.cmdline", "Command Line"),
                    ]),
                ],
                &[],
            ),
        ],
    },
    Tab {
        name: "Manage",
        panels: &[
            panel(
                "Customization",
                &[
                    Large((Icon::Gear, "ui.shortcuts", "User Interface")),
                    Column(&[(Icon::Gear, "aliasedit", "Edit Aliases"), (Icon::Gear, "options", "Options")]),
                ],
                &[],
            ),
            panel("Applications", &[Large((Icon::Import, "appload", "Load Application")), Column(&[(Icon::List, "script", "Run Script")])], &[]),
            panel(
                "Cleanup",
                &[Large((Icon::Erase, "purge", "Purge")), Column(&[(Icon::Join, "overkill", "Overkill"), (Icon::Search, "audit", "Audit")])],
                &[],
            ),
        ],
    },
    Tab {
        name: "Output",
        panels: &[
            panel(
                "Plot",
                &[
                    Large((Icon::Plot, "plot", "Plot")),
                    Column(&[
                        (Icon::PageSetup, "pagesetup", "Page Setup Manager"),
                        (Icon::Plot, "stylesmanager", "Plot Style Manager"),
                        (Icon::Preview, "preview", "Preview"),
                    ]),
                    Icons(&[(Icon::Publish, "publish", "Batch Plot")]),
                ],
                &[],
            ),
            panel("Export to DWF/PDF", &[Large((Icon::Export, "export", "Export"))], &[]),
        ],
    },
    Tab {
        name: "Add-ins",
        panels: &[panel("Applications", &[Large((Icon::Import, "appload", "Load Application")), Large((Icon::List, "script", "Run Script"))], &[])],
    },
    Tab {
        name: "Collaborate",
        panels: &[
            panel("Share", &[Large((Icon::Share, "share", "Share"))], &[]),
            panel("Compare", &[Large((Icon::Search, "compare", "DWG Compare"))], &[]),
        ],
    },
    Tab {
        name: "Express Tools",
        panels: &[
            panel(
                "Text",
                &[
                    Large((Icon::MText, "txt2mtxt", "Convert to Mtext")),
                    Column(&[(Icon::Text, "find", "Find"), (Icon::TextStyle, "style", "Text Style")]),
                ],
                &[],
            ),
            panel(
                "Modify",
                &[
                    Large((Icon::Join, "overkill", "Delete Duplicate Objects")),
                    Column(&[(Icon::Move, "align", "Align"), (Icon::Polyline, "pedit", "Edit Polyline")]),
                ],
                &[],
            ),
            panel(
                "Layers",
                &[
                    Large((Icon::LayerMatch, "laymrg", "Layer Merge")),
                    Column(&[(Icon::Erase, "laydel", "Layer Delete"), (Icon::LayerIso, "layiso", "Layer Isolate")]),
                ],
                &[],
            ),
            panel(
                "Tools",
                &[Large((Icon::List, "list", "List")), Column(&[(Icon::Point, "id", "ID Point"), (Icon::Area, "ui.toggle.quickcalc", "QuickCalc")])],
                &[],
            ),
        ],
    },
    Tab {
        name: "Featured Apps",
        panels: &[panel(
            "Apps",
            &[Large((Icon::Help, "ui.dialog.commands", "Command Reference")), Large((Icon::Help, "ui.dialog.about", "About"))],
            &[],
        )],
    },
];

/// Height of the tab row.
pub const TAB_H: f32 = 24.0;
/// Height of the panels under the tabs (buttons and panel titles).
pub const BODY_H: f32 = 92.0;
const ROW_H: f32 = 22.0;
const ROWS_TOP: f32 = 4.0;
const TITLE_H: f32 = 18.0;
const LARGE_ICON: f32 = 30.0;
const SMALL_ICON: f32 = 16.0;
const ICON_CELL: f32 = 24.0;
const ITEM_GAP: f32 = 4.0;
const PANEL_PAD: f32 = 6.0;
const LAYER_LIST_W: f32 = 232.0;
const PROPERTY_LISTS_W: f32 = 160.0;

/// The height the ribbon takes: the tab row, and the panels unless the ribbon is minimized.
pub fn height(app: &CadApp) -> f32 {
    // The Start page has no ribbon, as there is no drawing for its commands.
    if !app.ui.show_ribbon || app.ui.clean_screen.is_some() || app.ui.start_tab || app.session.docs.is_empty() {
        0.0
    } else if app.ui.ribbon_minimized {
        TAB_H
    } else {
        TAB_H + BODY_H
    }
}

fn command_known(id: &str) -> bool {
    autocraft_engine::find_command(id).is_some() || crate::menus::is_ui_command(id)
}

/// The tab named `name`, or the first.
fn tab(name: &str) -> &'static Tab {
    TABS.iter().find(|t| t.name == name).or(TABS.first()).unwrap_or(&EMPTY_TAB)
}

static EMPTY_TAB: Tab = Tab { name: "", panels: &[] };

/// A large button's label on one or two lines (split at the space nearest the middle).
fn large_lines(label: &str) -> Vec<String> {
    let label = crate::i18n::t(label);
    if label.chars().count() <= 9 || !label.contains(' ') {
        return vec![label.to_string()];
    }
    let mid = label.len() / 2;
    let split = label.match_indices(' ').map(|(i, _)| i).min_by_key(|i| i.abs_diff(mid));
    match split {
        Some(i) => vec![label[..i].to_string(), label[i + 1..].to_string()],
        None => vec![label.to_string()],
    }
}

fn text_w(p: &egui::Painter, text: &str) -> f32 {
    p.layout_no_wrap(text.to_string(), crate::theme::small(), Color32::WHITE).size().x
}

/// How much of a panel shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fit {
    /// Every button with its label.
    Full,
    /// Small buttons without their labels.
    Compact,
    /// One button that opens the panel's commands in a menu.
    Collapsed,
}

fn item_width(p: &egui::Painter, item: &Item, compact: bool) -> f32 {
    match item {
        Column(tools) if compact => tools.len().min(3).div_ceil(3) as f32 * icon_cell(tools),
        Large((_, _, label)) => large_lines(label).iter().map(|l| text_w(p, l)).fold(LARGE_ICON, f32::max) + 10.0,
        Column(tools) => {
            let arrows = if tools.iter().any(|t| variants(t.1).is_some()) { 12.0 } else { 0.0 };
            tools.iter().map(|(_, _, l)| text_w(p, crate::i18n::t(l))).fold(0.0, f32::max) + SMALL_ICON + 14.0 + arrows
        }
        Icons(tools) => tools.len().div_ceil(3) as f32 * icon_cell(tools),
        LayerList => LAYER_LIST_W,
        PropertyLists => PROPERTY_LISTS_W,
    }
}

/// Width of one small icon button in a grid: wider when the grid holds split buttons.
fn icon_cell(tools: &[Tool]) -> f32 {
    if tools.iter().any(|t| variants(t.1).is_some()) { ICON_CELL + 10.0 } else { ICON_CELL }
}

fn panel_width(p: &egui::Painter, panel: &Panel, compact: bool) -> f32 {
    let items: f32 = panel.items.iter().map(|i| item_width(p, i, compact)).sum::<f32>() + ITEM_GAP * panel.items.len().saturating_sub(1) as f32;
    (items + 2.0 * PANEL_PAD).max(text_w(p, crate::i18n::t(panel.name)) + 24.0)
}

fn collapsed_width(p: &egui::Painter, panel: &Panel) -> f32 {
    (text_w(p, crate::i18n::t(panel.name)) + 24.0).max(58.0)
}

/// How each panel shows in `room`: panels drop their small buttons' labels from the right, then
/// collapse from the right, until all of them fit. `widths` holds each panel's (full, compact,
/// collapsed) width.
fn fit(widths: &[(f32, f32, f32)], room: f32) -> Vec<Fit> {
    let mut fits = vec![Fit::Full; widths.len()];
    let width = |w: &(f32, f32, f32), f: Fit| match f {
        Fit::Full => w.0,
        Fit::Compact => w.1,
        Fit::Collapsed => w.2,
    };
    let total = |fits: &[Fit]| -> f32 { widths.iter().zip(fits).map(|(w, f)| width(w, *f)).sum() };
    for step in [Fit::Compact, Fit::Collapsed] {
        for i in (0..widths.len()).rev() {
            if total(&fits) <= room {
                return fits;
            }
            if let Some(f) = fits.get_mut(i) {
                *f = step;
            }
        }
    }
    fits
}

/// The variants of a split button whose command is `cmd`.
fn variants(cmd: &str) -> Option<&'static [Tool]> {
    SPLITS.iter().find(|(main, _)| *main == cmd).map(|(_, v)| *v)
}

/// The tool a split button runs now: the variant chosen last, else `tool`.
fn current(app: &CadApp, tool: Tool) -> Tool {
    let Some(list) = variants(tool.1) else { return tool };
    let chosen = app.ui.ribbon_last.iter().find(|(main, _)| main == tool.1).map(|(_, v)| v.as_str());
    match chosen.and_then(|c| list.iter().find(|v| v.1 == c)) {
        // The button keeps its own name; the icon and command follow the variant.
        Some(v) => (v.0, v.1, tool.2),
        None => tool,
    }
}

/// Remember `variant` as split button `main`'s command.
fn choose(app: &mut CadApp, main: &str, variant: &str) {
    app.ui.ribbon_last.retain(|(m, _)| m != main);
    if main != variant {
        app.ui.ribbon_last.push((main.to_string(), variant.to_string()));
    }
}

/// A small down arrow centred at `c`.
fn arrow(p: &egui::Painter, c: egui::Pos2, color: Color32) {
    p.add(egui::Shape::convex_polygon(vec![c + vec2(-3.0, -1.5), c + vec2(3.0, -1.5), c + vec2(0.0, 2.0)], color, Stroke::NONE));
}

/// The menu of a split button's variants under `resp`; a choice runs and is remembered.
fn split_menu(app: &mut CadApp, resp: &egui::Response, main: &'static str, clicked: &mut Option<&'static str>) {
    let Some(list) = variants(main) else { return };
    let mut pick = None;
    egui::Popup::menu(resp).show(|ui| {
        ui.set_min_width(170.0);
        for (icon, cmd, label) in list {
            let known = command_known(cmd);
            let r = ui.add_enabled(known, egui::Button::new(format!("      {}", crate::i18n::t(label))));
            icons::paint(ui.painter(), Rect::from_center_size(pos2(r.rect.left() + 10.0, r.rect.center().y), vec2(14.0, 14.0)), *icon, !known);
            icons::describe_control(ui, &r, label, None, false);
            if r.clicked() {
                pick = Some(*cmd);
                ui.close();
            }
        }
    });
    if let Some(cmd) = pick {
        choose(app, main, cmd);
        *clicked = Some(cmd);
    }
}

/// Tooltip text for a tool: its label and the command a user can type.
fn tip(cmd: &str, label: &str) -> String {
    if cmd.starts_with("ui.") { crate::i18n::t(label).to_string() } else { format!("{}\n{}", crate::i18n::t(label), cmd.to_ascii_uppercase()) }
}

/// A clickable area with the ribbon's hover look; disabled when its command is unknown.
fn hot(ui: &egui::Ui, rect: Rect, id: egui::Id, cmd: &str, label: &str) -> (egui::Response, bool) {
    let t = Tokens::get();
    let known = command_known(cmd);
    let sense = if known { Sense::click() } else { Sense::hover() };
    let resp = ui.interact(rect, id, sense);
    if known && resp.hovered() {
        ui.painter().rect_filled(rect, 2.0, t.control_hover.gamma_multiply(0.55));
        ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.0, t.control_hover), egui::StrokeKind::Inside);
    }
    icons::describe_control(ui, &resp, label, None, false);
    (resp, known)
}

/// The ribbon panel (top of the window, below the quick access bar).
pub fn show(app: &mut CadApp, ui: &mut egui::Ui) {
    let h = height(app);
    if h <= 0.0 {
        return;
    }
    let t = Tokens::get();
    let mut clicked: Option<&'static str> = None;
    egui::Panel::top("cc_ribbon").exact_size(h).frame(egui::Frame::NONE.fill(t.chrome)).show(ui, |ui| {
        let r = ui.max_rect();
        let p = ui.painter().clone();
        // Tab row.
        let row = Rect::from_min_size(r.min, vec2(r.width(), TAB_H));
        p.rect_filled(row, 0.0, t.title_bar);
        let mut x = row.left() + 6.0;
        let current = tab(&app.ui.ribbon_tab).name;
        let mut pick = None;
        for tb in TABS {
            let label = crate::i18n::t(tb.name);
            let w = text_w(&p, label).max(text_w(&p, tb.name)) + 20.0;
            let tr = Rect::from_min_size(pos2(x, row.top() + 2.0), vec2(w, TAB_H - 2.0));
            let resp = ui.interact(tr, ui.scope_id().with(("ribbon_tab", tb.name)), Sense::click());
            let active = tb.name == current;
            if active && !app.ui.ribbon_minimized {
                p.rect_filled(tr, egui::CornerRadius { nw: 3, ne: 3, sw: 0, se: 0 }, t.chrome);
            } else if active {
                p.rect_filled(tr, 3.0, t.control.gamma_multiply(0.6));
            } else if resp.hovered() {
                p.rect_filled(tr, 3.0, t.control.gamma_multiply(0.45));
            }
            p.text(tr.center(), Align2::CENTER_CENTER, label, crate::theme::body(), if active { t.text } else { t.text_dim });
            icons::describe_control(ui, &resp, tb.name, Some(active), true);
            if resp.clicked() {
                pick = Some(tb.name);
            }
            if resp.double_clicked() {
                app.ui.ribbon_minimized = !app.ui.ribbon_minimized;
            }
            x += w + 2.0;
        }
        // The minimize button after the tabs.
        let mr = Rect::from_min_size(pos2(x + 6.0, row.top() + 4.0), vec2(20.0, TAB_H - 8.0));
        let mresp = ui.interact(mr, ui.scope_id().with("ribbon_min"), Sense::click());
        if mresp.hovered() {
            p.rect_filled(mr, 2.0, t.control_hover.gamma_multiply(0.6));
        }
        icons::paint(&p, mr.shrink(2.0), if app.ui.ribbon_minimized { Icon::ChevronDown } else { Icon::ChevronUp }, false);
        let mtip = if app.ui.ribbon_minimized { "Show Full Ribbon" } else { "Minimize Ribbon" };
        icons::describe_control(ui, &mresp, mtip, None, false);
        if mresp.on_hover_text(crate::i18n::t(mtip)).clicked() {
            app.ui.ribbon_minimized = !app.ui.ribbon_minimized;
        }
        if let Some(name) = pick {
            if app.ui.ribbon_minimized && name == current {
                app.ui.ribbon_minimized = false;
            }
            app.ui.ribbon_tab = name.to_string();
        }
        if app.ui.ribbon_minimized {
            return;
        }
        // Panels.
        let body = Rect::from_min_max(pos2(r.left(), row.bottom()), r.max);
        p.hline(body.x_range(), body.bottom() - 0.5, Stroke::new(1.0, t.border));
        let panels = tab(&app.ui.ribbon_tab).panels;
        let widths: Vec<(f32, f32, f32)> =
            panels.iter().map(|pn| (panel_width(&p, pn, false), panel_width(&p, pn, true), collapsed_width(&p, pn))).collect();
        let fits = fit(&widths, (body.width() - 8.0).max(0.0));
        let mut x = body.left() + 4.0;
        for ((panel, f), w) in panels.iter().zip(fits).zip(&widths) {
            let w = match f {
                Fit::Full => w.0,
                Fit::Compact => w.1,
                Fit::Collapsed => w.2,
            };
            let pr = Rect::from_min_size(pos2(x, body.top() + 3.0), vec2(w, body.height() - 6.0));
            match f {
                Fit::Collapsed => draw_collapsed(app, ui, &p, panel, pr, &mut clicked),
                _ => draw_panel(app, ui, &p, panel, pr, f == Fit::Compact, &mut clicked),
            }
            x += w;
            p.vline(x, pr.y_range().shrink(4.0), Stroke::new(1.0, t.border));
            x += 1.0;
        }
    });
    if let Some(cmd) = clicked {
        crate::menus::activate(app, cmd);
    }
}

/// The title strip under a panel's buttons.
fn panel_title(p: &egui::Painter, pr: Rect, name: &str, chevron: bool) {
    let t = Tokens::get();
    let title = Rect::from_min_max(pos2(pr.left() + 1.0, pr.bottom() - TITLE_H), pos2(pr.right() - 1.0, pr.bottom()));
    p.rect_filled(title, 2.0, t.chrome_dark);
    let label = crate::i18n::t(name);
    let g = p.layout_no_wrap(label.to_string(), crate::theme::small(), t.text_dim);
    let cx = title.center().x - if chevron { 5.0 } else { 0.0 };
    let gw = g.size().x;
    p.galley(pos2(cx - gw / 2.0, title.center().y - g.size().y / 2.0), g, t.text_dim);
    if chevron {
        let c = pos2(cx + gw / 2.0 + 7.0, title.center().y);
        p.add(egui::Shape::convex_polygon(vec![c + vec2(-3.0, -1.5), c + vec2(3.0, -1.5), c + vec2(0.0, 2.0)], t.text_dim, Stroke::NONE));
    }
}

fn draw_panel(app: &mut CadApp, ui: &mut egui::Ui, p: &egui::Painter, panel: &Panel, pr: Rect, compact: bool, clicked: &mut Option<&'static str>) {
    let t = Tokens::get();
    let mut x = pr.left() + PANEL_PAD;
    let top = pr.top() + ROWS_TOP;
    for (k, item) in panel.items.iter().enumerate() {
        let w = item_width(p, item, compact);
        let id = ui.scope_id().with(("ribbon", panel.name, k));
        match item {
            Large(tool) => {
                let main = tool.1;
                let (icon, cmd, label) = current(app, *tool);
                let br = Rect::from_min_size(pos2(x, top), vec2(w, 3.0 * ROW_H));
                let split = variants(main).is_some();
                // A split button: the icon runs the command, the label and arrow open the variants.
                let top_r = if split { Rect::from_min_max(br.min, pos2(br.right(), br.top() + LARGE_ICON + 6.0)) } else { br };
                let (resp, known) = hot(ui, top_r, id, cmd, label);
                let ir = Rect::from_center_size(pos2(br.center().x, br.top() + 3.0 + LARGE_ICON / 2.0), vec2(LARGE_ICON, LARGE_ICON));
                icons::paint(p, ir, icon, !known);
                let mut y = ir.bottom() + 2.0;
                let lines = large_lines(label);
                let n = lines.len();
                for (k, line) in lines.into_iter().enumerate() {
                    let g = p.layout_no_wrap(line, crate::theme::small(), if known { t.text } else { t.text_faint });
                    let gx = br.center().x - g.size().x / 2.0 - if split && k + 1 == n { 4.0 } else { 0.0 };
                    let gw = g.size().x;
                    p.galley(pos2(gx, y), g, if known { t.text } else { t.text_faint });
                    if split && k + 1 == n {
                        arrow(p, pos2(gx + gw + 6.0, y + 7.0), t.text_dim);
                    }
                    y += 13.0;
                }
                if resp.on_hover_text(tip(cmd, label)).clicked() {
                    *clicked = Some(cmd);
                }
                if split {
                    let bottom = Rect::from_min_max(pos2(br.left(), top_r.bottom()), br.max);
                    let (more, _) = hot(ui, bottom, id.with("split"), main, &format!("{label} options"));
                    split_menu(app, &more, main, clicked);
                }
            }
            Column(tools) if !compact => {
                for (row, tool) in tools.iter().take(3).enumerate() {
                    let main = tool.1;
                    let (icon, cmd, label) = current(app, *tool);
                    let full = Rect::from_min_size(pos2(x, top + row as f32 * ROW_H), vec2(w, ROW_H));
                    let split = variants(main).is_some();
                    let br = if split { Rect::from_min_max(full.min, pos2(full.right() - 12.0, full.bottom())) } else { full };
                    if split {
                        let ar = Rect::from_min_max(pos2(br.right(), full.top()), full.max);
                        let (more, _) = hot(ui, ar, id.with(("split", row)), main, &format!("{label} options"));
                        arrow(p, ar.center(), t.text_dim);
                        split_menu(app, &more, main, clicked);
                    }
                    let (resp, known) = hot(ui, br, id.with(row), cmd, label);
                    let ir = Rect::from_center_size(pos2(br.left() + 3.0 + SMALL_ICON / 2.0, br.center().y), vec2(SMALL_ICON, SMALL_ICON));
                    icons::paint(p, ir, icon, !known);
                    p.text(
                        pos2(ir.right() + 5.0, br.center().y),
                        Align2::LEFT_CENTER,
                        crate::i18n::t(label),
                        crate::theme::small(),
                        if known { t.text } else { t.text_faint },
                    );
                    if resp.on_hover_text(tip(cmd, label)).clicked() {
                        *clicked = Some(cmd);
                    }
                }
            }
            Icons(tools) | Column(tools) => {
                let cell = icon_cell(tools);
                for (n, tool) in tools.iter().enumerate() {
                    let main = tool.1;
                    let (icon, cmd, label) = current(app, *tool);
                    let (col, row) = (n / 3, n % 3);
                    let full = Rect::from_min_size(pos2(x + col as f32 * cell, top + row as f32 * ROW_H), vec2(cell - 2.0, ROW_H));
                    let split = variants(main).is_some();
                    let br = if split { Rect::from_min_max(full.min, pos2(full.right() - 10.0, full.bottom())) } else { full };
                    if split {
                        let ar = Rect::from_min_max(pos2(br.right(), full.top()), full.max);
                        let (more, _) = hot(ui, ar, id.with(("split", n)), main, &format!("{label} options"));
                        arrow(p, ar.center(), t.text_dim);
                        split_menu(app, &more, main, clicked);
                    }
                    let (resp, known) = hot(ui, br, id.with(n), cmd, label);
                    icons::paint(p, Rect::from_center_size(br.center(), vec2(SMALL_ICON, SMALL_ICON)), icon, !known);
                    if resp.on_hover_text(tip(cmd, label)).clicked() {
                        *clicked = Some(cmd);
                    }
                }
            }
            LayerList => layer_list(app, ui, p, Rect::from_min_size(pos2(x, top), vec2(w, 3.0 * ROW_H)), clicked),
            PropertyLists => property_lists(app, ui, p, Rect::from_min_size(pos2(x, top), vec2(w, 3.0 * ROW_H)), clicked),
        }
        x += w + ITEM_GAP;
    }
    slide_out(app, ui, p, panel, pr, clicked);
}

/// The panel's title strip; with a slide-out, the title opens it under the panel.
fn slide_out(app: &mut CadApp, ui: &mut egui::Ui, p: &egui::Painter, panel: &Panel, pr: Rect, clicked: &mut Option<&'static str>) {
    panel_title(p, pr, panel.name, !panel.more.is_empty());
    if panel.more.is_empty() {
        return;
    }
    let title = Rect::from_min_max(pos2(pr.left() + 1.0, pr.bottom() - TITLE_H), pos2(pr.right() - 1.0, pr.bottom()));
    let resp = ui.interact(title, ui.scope_id().with(("ribbon_slide", panel.name)), Sense::click());
    if resp.hovered() {
        p.rect_stroke(title, 2.0, Stroke::new(1.0, Tokens::get().control_hover), egui::StrokeKind::Inside);
    }
    icons::describe_control(ui, &resp, &format!("{} panel", panel.name), None, false);
    let mut pick = None;
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(pr.width().max(180.0));
        for (icon, cmd, label) in panel.more {
            let known = command_known(cmd);
            let r = ui.add_enabled(known, egui::Button::new(format!("      {}", crate::i18n::t(label))));
            icons::paint(ui.painter(), Rect::from_center_size(pos2(r.rect.left() + 12.0, r.rect.center().y), vec2(15.0, 15.0)), *icon, !known);
            icons::describe_control(ui, &r, label, None, false);
            if r.clicked() {
                pick = Some(*cmd);
                ui.close();
            }
        }
    });
    if let Some(cmd) = pick {
        *clicked = Some(cmd);
    }
    let _ = app;
}

/// A panel too wide for the window: one button with the panel's first icon that opens all of its
/// commands in a menu.
fn draw_collapsed(_app: &mut CadApp, ui: &mut egui::Ui, p: &egui::Painter, panel: &Panel, pr: Rect, clicked: &mut Option<&'static str>) {
    let t = Tokens::get();
    let br = Rect::from_min_max(pr.min + vec2(2.0, ROWS_TOP), pos2(pr.right() - 2.0, pr.bottom() - TITLE_H - 2.0));
    let resp = ui.interact(br, ui.scope_id().with(("ribbon_more", panel.name)), Sense::click());
    if resp.hovered() {
        p.rect_filled(br, 2.0, t.control_hover.gamma_multiply(0.55));
    }
    let icon = panel.items.iter().find_map(|i| match i {
        Large((icon, _, _)) => Some(*icon),
        Column(tools) | Icons(tools) => tools.first().map(|(icon, _, _)| *icon),
        LayerList => Some(Icon::LayerProps),
        PropertyLists => Some(Icon::MatchProp),
    });
    if let Some(icon) = icon {
        icons::paint(p, Rect::from_center_size(br.center() - vec2(0.0, 6.0), vec2(LARGE_ICON, LARGE_ICON)), icon, false);
    }
    let c = pos2(br.center().x, br.bottom() - 7.0);
    p.add(egui::Shape::convex_polygon(vec![c + vec2(-3.5, -2.0), c + vec2(3.5, -2.0), c + vec2(0.0, 2.0)], t.text_dim, Stroke::NONE));
    icons::describe_control(ui, &resp, panel.name, None, false);
    egui::Popup::menu(&resp).show(|ui| {
        for item in panel.items {
            let tools: Vec<Tool> = match item {
                Large(tool) => vec![*tool],
                Column(tools) | Icons(tools) => tools.to_vec(),
                LayerList => vec![(Icon::LayerProps, "ui.dialog.layers", "Layer Properties")],
                PropertyLists => vec![(Icon::Properties, "ui.panel.properties", "Properties")],
            };
            for (_, cmd, label) in tools.into_iter().chain(panel.more.iter().copied()) {
                let b = ui.add_enabled(command_known(cmd), egui::Button::new(crate::i18n::t(label)));
                crate::control::record_widget(ui, &b, label, "button", None);
                if b.clicked() {
                    *clicked = Some(cmd);
                    ui.close();
                }
            }
        }
    });
    panel_title(p, pr, panel.name, true);
}

/// A drop-down box drawn like the ribbon's lists.
fn list_box(ui: &egui::Ui, p: &egui::Painter, rect: Rect, id: egui::Id, label: &str) -> egui::Response {
    let t = Tokens::get();
    let resp = ui.interact(rect, id, Sense::click());
    p.rect_filled(rect, 2.0, if resp.hovered() { t.control_hover } else { t.field });
    p.rect_stroke(rect, 2.0, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    let c = pos2(rect.right() - 9.0, rect.center().y);
    p.add(egui::Shape::convex_polygon(vec![c + vec2(-3.5, -2.0), c + vec2(3.5, -2.0), c + vec2(0.0, 2.0)], t.text_dim, Stroke::NONE));
    icons::describe_control(ui, &resp, label, None, false);
    resp
}

fn swatch(p: &egui::Painter, r: Rect, c: Color) {
    let rgb = c.resolve(Color::Index(7), Color::Index(7));
    p.rect_filled(r, 1.0, Color32::from_rgb(rgb.0, rgb.1, rgb.2));
    p.rect_stroke(r, 1.0, Stroke::new(1.0, Color32::from_gray(30)), egui::StrokeKind::Inside);
}

/// The current layer list over two rows of layer tools.
fn layer_list(app: &mut CadApp, ui: &mut egui::Ui, p: &egui::Painter, area: Rect, clicked: &mut Option<&'static str>) {
    let t = Tokens::get();
    let Ok(d) = app.session.doc() else { return };
    let cur = d.header.str("CLAYER", "0");
    let layers: Vec<(String, bool, bool, bool, Color)> = d.layers.iter().map(|l| (l.name.clone(), l.on, l.frozen, l.locked, l.color)).collect();
    let info = layers.iter().find(|l| l.0.eq_ignore_ascii_case(&cur)).cloned();
    let combo = Rect::from_min_size(area.min + vec2(0.0, 1.0), vec2(area.width(), ROW_H - 2.0));
    let resp = list_box(ui, p, combo, ui.scope_id().with("ribbon_layer"), "Layer");
    if let Some((name, on, frozen, locked, color)) = &info {
        let mut x = combo.left() + 4.0;
        for icon in [
            if *on { Icon::Bulb } else { Icon::BulbOff },
            if *frozen { Icon::Snowflake } else { Icon::Sun },
            if *locked { Icon::Lock } else { Icon::Unlock },
        ] {
            icons::paint(p, Rect::from_min_size(pos2(x, combo.top() + 2.0), vec2(15.0, 15.0)), icon, false);
            x += 16.0;
        }
        swatch(p, Rect::from_min_size(pos2(x + 2.0, combo.center().y - 5.0), vec2(10.0, 10.0)), *color);
        p.with_clip_rect(combo.shrink(2.0)).text(pos2(x + 18.0, combo.center().y), Align2::LEFT_CENTER, name, crate::theme::small(), t.text);
    }
    let mut set_current = None;
    let mut toggle: Option<(String, &str, bool)> = None;
    egui::Popup::from_toggle_button_response(&resp).width(area.width()).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside).show(|ui| {
        egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
            for (name, on, frozen, locked, color) in &layers {
                ui.horizontal(|ui| {
                    if icons::button(ui, if *on { Icon::Bulb } else { Icon::BulbOff }, 18.0, "On/Off", false).clicked() {
                        toggle = Some((name.clone(), "on", !*on));
                    }
                    if icons::button(ui, if *frozen { Icon::Snowflake } else { Icon::Sun }, 18.0, "Freeze/Thaw", false).clicked() {
                        toggle = Some((name.clone(), "frozen", !*frozen));
                    }
                    if icons::button(ui, if *locked { Icon::Lock } else { Icon::Unlock }, 18.0, "Lock/Unlock", false).clicked() {
                        toggle = Some((name.clone(), "locked", !*locked));
                    }
                    let (sr, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                    swatch(ui.painter(), sr, *color);
                    if ui.selectable_label(name.eq_ignore_ascii_case(&cur), name).clicked() {
                        set_current = Some(name.clone());
                        ui.close();
                    }
                });
            }
        });
    });
    // Two rows of layer tools under the list.
    const TOOLS: &[Tool] = &[
        (Icon::LayerOff, "layoff", "Off"),
        (Icon::LayerIso, "layiso", "Isolate"),
        (Icon::LayerFreeze, "layfrz", "Freeze"),
        (Icon::LayerLock, "laylck", "Lock"),
        (Icon::Unlock, "layulk", "Unlock"),
        (Icon::LayerPrev, "layerp", "Previous"),
        (Icon::Bulb, "layon", "Turn All Layers On"),
        (Icon::Sun, "laythw", "Thaw All Layers"),
        (Icon::LayerIso, "layuniso", "Unisolate"),
    ];
    for (n, (icon, cmd, label)) in TOOLS.iter().enumerate() {
        let (row, col) = (1 + n / 5, n % 5);
        let br = Rect::from_min_size(pos2(area.left() + col as f32 * ICON_CELL, area.top() + row as f32 * ROW_H), vec2(ICON_CELL - 2.0, ROW_H));
        let (resp, known) = hot(ui, br, ui.scope_id().with(("ribbon_layer_tool", n)), cmd, label);
        icons::paint(p, Rect::from_center_size(br.center(), vec2(SMALL_ICON, SMALL_ICON)), *icon, !known);
        if resp.on_hover_text(tip(cmd, label)).clicked() {
            *clicked = Some(cmd);
        }
    }
    // Make Current and Match Layer, labelled, beside the tools.
    let left = area.left() + 5.0 * ICON_CELL + 2.0;
    for (row, (icon, cmd, label)) in
        [(Icon::MakeCurrent, "laymcur", "Make Current"), (Icon::LayerMatch, "laymch", "Match Layer")].into_iter().enumerate()
    {
        let br = Rect::from_min_max(pos2(left, area.top() + (row + 1) as f32 * ROW_H), pos2(area.right(), area.top() + (row + 2) as f32 * ROW_H));
        let (resp, known) = hot(ui, br, ui.scope_id().with(("ribbon_layer_cmd", row)), cmd, label);
        let ir = Rect::from_center_size(pos2(br.left() + 3.0 + SMALL_ICON / 2.0, br.center().y), vec2(SMALL_ICON, SMALL_ICON));
        icons::paint(p, ir, icon, !known);
        p.text(pos2(ir.right() + 4.0, br.center().y), Align2::LEFT_CENTER, crate::i18n::t(label), crate::theme::small(), t.text);
        if resp.on_hover_text(tip(cmd, label)).clicked() {
            *clicked = Some(cmd);
        }
    }
    if let Some(n) = set_current {
        let _ = app.run("layer.current", json!({ "name": n }));
    }
    if let Some((n, k, v)) = toggle {
        let _ = app.run("layer.set", json!({ "name": n, k: v }));
    }
}

/// The colour, linetype and lineweight given to new objects; each list starts its command.
fn property_lists(app: &mut CadApp, ui: &mut egui::Ui, p: &egui::Painter, area: Rect, clicked: &mut Option<&'static str>) {
    let t = Tokens::get();
    let Ok(d) = app.session.doc() else { return };
    let color = Color::from_aci(i16::try_from(d.header.i64("CECOLOR", 256)).unwrap_or(256));
    let linetype = d.header.str("CELTYPE", "ByLayer");
    let weight = autocraft_doc::Lineweight::from_dxf(i16::try_from(d.header.i64("CELWEIGHT", -1)).unwrap_or(-1)).name();
    let rows: [(&'static str, &str, String); 3] =
        [("color", "Object Color", color.name()), ("linetype", "Linetype", linetype), ("lweight", "Lineweight", weight.to_string())];
    for (row, (cmd, label, value)) in rows.into_iter().enumerate() {
        let r = Rect::from_min_size(pos2(area.left(), area.top() + row as f32 * ROW_H + 1.0), vec2(area.width(), ROW_H - 2.0));
        let ir = Rect::from_center_size(pos2(r.left() + 9.0, r.center().y), vec2(14.0, 14.0));
        let field = Rect::from_min_max(pos2(r.left() + 20.0, r.top()), r.max);
        let resp = list_box(ui, p, field, ui.scope_id().with(("ribbon_prop", row)), label);
        let mut text_x = field.left() + 6.0;
        match row {
            0 => {
                swatch(p, Rect::from_center_size(ir.center(), vec2(11.0, 11.0)), color);
                swatch(p, Rect::from_min_size(pos2(text_x, field.center().y - 5.0), vec2(10.0, 10.0)), color);
                text_x += 15.0;
            }
            1 => {
                for k in 0..3 {
                    let y = ir.top() + 3.0 + k as f32 * 4.0;
                    p.hline(ir.x_range(), y, Stroke::new(1.0, t.icon));
                }
                p.hline(pos2(text_x, 0.0).x..=text_x + 28.0, field.center().y, Stroke::new(1.0, t.text));
                text_x += 34.0;
            }
            _ => {
                for (k, w) in [1.0, 2.0, 3.0].into_iter().enumerate() {
                    p.hline(ir.x_range(), ir.top() + 3.0 + k as f32 * 4.5, Stroke::new(w, t.icon));
                }
                p.hline(pos2(text_x, 0.0).x..=text_x + 28.0, field.center().y, Stroke::new(2.0, t.text));
                text_x += 34.0;
            }
        }
        p.with_clip_rect(field.shrink(2.0)).text(pos2(text_x, field.center().y), Align2::LEFT_CENTER, value, crate::theme::small(), t.text);
        if resp.on_hover_text(crate::i18n::t(label)).clicked() && command_known(cmd) {
            *clicked = Some(cmd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_ribbon_command_exists() {
        let mut missing = Vec::new();
        for tab in TABS {
            for panel in tab.panels {
                for item in panel.items {
                    let tools: Vec<Tool> = match item {
                        Large(tool) => vec![*tool],
                        Column(tools) | Icons(tools) => tools.to_vec(),
                        LayerList | PropertyLists => Vec::new(),
                    };
                    missing.extend(tools.iter().filter(|(_, c, _)| !command_known(c)).map(|(_, c, _)| format!("{}/{}: {c}", tab.name, panel.name)));
                }
                for (_, c, _) in panel.more {
                    if !command_known(c) {
                        missing.push(format!("{}/{} slide-out: {c}", tab.name, panel.name));
                    }
                }
            }
        }
        for (main, list) in SPLITS {
            missing.extend(list.iter().filter(|(_, c, _)| !command_known(c)).map(|(_, c, _)| format!("split {main}: {c}")));
        }
        assert!(missing.is_empty(), "unknown ribbon commands: {missing:?}");
    }

    #[test]
    fn panels_collapse_from_the_right_until_they_fit() {
        let w = [(100.0, 80.0, 50.0); 3];
        assert_eq!(fit(&w, 300.0), vec![Fit::Full; 3]);
        assert_eq!(fit(&w, 280.0), vec![Fit::Full, Fit::Full, Fit::Compact]);
        assert_eq!(fit(&w, 240.0), vec![Fit::Compact; 3]);
        assert_eq!(fit(&w, 210.0), vec![Fit::Compact, Fit::Compact, Fit::Collapsed]);
        assert_eq!(fit(&w, 10.0), vec![Fit::Collapsed; 3]);
    }

    #[test]
    fn long_large_labels_wrap_onto_two_lines() {
        assert_eq!(large_lines("Line"), vec!["Line".to_string()]);
        assert_eq!(large_lines("Layer Properties"), vec!["Layer".to_string(), "Properties".to_string()]);
        assert_eq!(large_lines("Match Properties"), vec!["Match".to_string(), "Properties".to_string()]);
    }
}
