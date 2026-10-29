//! Ribbon widget (iced) — patrón `ui-patterns/ribbon-ui` (ver protocolo SSOT en
//! better-toolkits). Tab bar + tool area de grupos + colapso, anatomia
//! OpenCADStudio/Office.

use iced::widget::{button, container, text};
use iced::widget::{Column, Row};
use iced::{Element, Length};

/// Mensajes emitidos por el Ribbon.
#[derive(Debug, Clone)]
pub enum RibbonMessage {
    SelectTab(usize),
    ToggleCollapse,
    Command(&'static str),
}

/// Un comando de la tool area (label + tooltip).
#[derive(Debug, Clone)]
pub struct RibbonCommand {
    pub label: &'static str,
    pub tip: &'static str,
}

/// Un grupo de la tool area (título + comandos).
#[derive(Debug, Clone)]
pub struct RibbonGroupSpec {
    pub title: &'static str,
    pub commands: Vec<RibbonCommand>,
}

/// Estado del Ribbon.
#[derive(Debug, Clone, Default)]
pub struct RibbonState {
    pub active: usize,
    pub collapsed: bool,
}

impl RibbonState {
    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed;
    }
    pub fn select_tab(&mut self, i: usize) {
        self.active = i;
    }
}

/// Construye la vista del Ribbon.
pub fn ribbon_view<'a>(
    tabs: &'a [&'static str],
    state: &RibbonState,
    groups: &'a [RibbonGroupSpec],
) -> Element<'a, RibbonMessage> {
    // Tab bar
    let mut tab_row = Row::new().spacing(8);
    for (i, t) in tabs.iter().enumerate() {
        tab_row = tab_row.push(button(text(*t)).on_press(RibbonMessage::SelectTab(i)));
    }
    let collapse_btn = button(text(if state.collapsed { "▼" } else { "▲" }))
        .on_press(RibbonMessage::ToggleCollapse);
    let header = Row::new().push(tab_row).push(collapse_btn).spacing(16);

    if state.collapsed {
        return container(header).width(Length::Fill).into();
    }

    // Tool area: un grupo = columna de comandos + título.
    let mut tool_area = Row::new().spacing(20);
    for g in groups {
        let mut cmds = Column::new().spacing(4);
        for c in &g.commands {
            cmds = cmds.push(button(text(c.label)).on_press(RibbonMessage::Command(c.label)));
        }
        let group = Column::new().push(cmds).push(text(g.title)).spacing(4);
        tool_area = tool_area.push(group);
    }

    container(Column::new().push(header).push(tool_area).spacing(8))
        .width(Length::Fill)
        .into()
}
