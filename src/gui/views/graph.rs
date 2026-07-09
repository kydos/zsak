use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Element, Length};

use crate::app::{AppState, Message};
use crate::widgets::graph_canvas::{GraphCanvas, GraphState, NodeKind, RenderedEdge, RenderedNode};

#[derive(Clone, Debug)]
pub struct GraphData {
    pub dot: String,
    pub graph_state: GraphState,
}

/// Parse a DOT string produced by the Zenoh routing graph query.
///
/// Zenoh emits something like:
///   digraph {
///     "abc123" [label="abc...|Router", shape=record]
///     "def456" [label="def...|Peer"]
///     "abc123" -> "def456"
///   }
/// or with unquoted IDs, `--` edges (strict graph), etc.
pub fn parse_dot(dot: &str) -> GraphData {
    use std::collections::HashMap;

    let mut nodes: Vec<RenderedNode> = Vec::new();
    let mut edges: Vec<RenderedEdge> = Vec::new();
    let mut node_index: HashMap<String, usize> = HashMap::new();

    for line in dot.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }

        // Detect edge lines: contain "->" or "--"
        let is_edge = line.contains("->") || line.contains(" -- ");
        let edge_sep = if line.contains("->") { "->" } else { " -- " };

        if is_edge {
            let parts: Vec<&str> = line.splitn(2, edge_sep).collect();
            if parts.len() == 2 {
                let from_id = extract_id(parts[0].trim());
                // strip trailing attributes from the "to" part
                let to_raw = parts[1].trim().splitn(2, '[').next().unwrap_or("").trim();
                let to_id = extract_id(to_raw);
                if let (Some(from), Some(to)) = (from_id, to_id) {
                    let from_idx = get_or_insert_node(&mut nodes, &mut node_index, &from);
                    let to_idx = get_or_insert_node(&mut nodes, &mut node_index, &to);
                    let label = extract_attr(line, "label").unwrap_or_default();
                    edges.push(RenderedEdge { from: from_idx, to: to_idx, label });
                }
            }
            continue;
        }

        // Node definition: has an ID followed by optional [attrs] or ;
        // Matches: "id" [...] or id [...] or "id"; or id;
        let id = match extract_id(line) {
            Some(id) if !id.is_empty()
                && id != "digraph"
                && id != "strict"
                && id != "graph"
                && id != "node"
                && id != "edge"
                && !line.trim_start().starts_with('{')
                && !line.trim_start().starts_with('}') => id,
            _ => continue,
        };

        // Skip lines that are just graph keywords
        let kind = classify_kind(line);
        let label = extract_attr(line, "label")
            .map(|l| {
                // Zenoh record labels look like "abc123...|Router" — take the first part
                let part = l.split('|').next().unwrap_or(&l).trim();
                shorten_zid(part)
            })
            .unwrap_or_else(|| shorten_zid(&id));

        if !node_index.contains_key(&id) {
            let idx = nodes.len();
            node_index.insert(id.clone(), idx);
            nodes.push(RenderedNode { id, label, kind, pos: (0.0, 0.0) });
        } else {
            // Update label/kind if we already inserted this node from an edge
            if let Some(&idx) = node_index.get(&id) {
                nodes[idx].label = label;
                nodes[idx].kind = kind;
            }
        }
    }

    // Circular layout
    let n = nodes.len();
    if n > 0 {
        let radius = 200.0_f32;
        for (i, node) in nodes.iter_mut().enumerate() {
            let angle = 2.0 * std::f32::consts::PI * (i as f32) / (n as f32);
            node.pos = (radius * angle.cos(), radius * angle.sin());
        }
    }

    GraphData {
        dot: dot.to_string(),
        graph_state: GraphState { nodes, edges },
    }
}

fn classify_kind(line: &str) -> NodeKind {
    let lower = line.to_lowercase();
    if lower.contains("router") {
        NodeKind::Router
    } else if lower.contains("peer") {
        NodeKind::Peer
    } else {
        NodeKind::Client
    }
}

fn get_or_insert_node(
    nodes: &mut Vec<RenderedNode>,
    index: &mut std::collections::HashMap<String, usize>,
    id: &str,
) -> usize {
    if let Some(&idx) = index.get(id) {
        idx
    } else {
        let idx = nodes.len();
        index.insert(id.to_string(), idx);
        nodes.push(RenderedNode {
            id: id.to_string(),
            label: shorten_zid(id),
            kind: NodeKind::Peer,
            pos: (0.0, 0.0),
        });
        idx
    }
}

/// Extract a node/edge ID from the start of a line.
/// Handles both quoted ("abc") and unquoted (abc) IDs.
fn extract_id(s: &str) -> Option<String> {
    let s = s.trim();
    if s.starts_with('"') {
        // Quoted ID
        let inner = &s[1..];
        let end = inner.find('"')?;
        Some(inner[..end].to_string())
    } else {
        // Unquoted: take until whitespace, '[', ';', '{', '}'
        let end = s.find(|c: char| c.is_whitespace() || matches!(c, '[' | ';' | '{' | '}'))
            .unwrap_or(s.len());
        let id = &s[..end];
        if id.is_empty() { None } else { Some(id.to_string()) }
    }
}

fn extract_attr(s: &str, attr: &str) -> Option<String> {
    // Try quoted: attr="value"
    let needle = format!("{}=\"", attr);
    if let Some(start) = s.find(&needle) {
        let rest = &s[start + needle.len()..];
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    // Try unquoted: attr=value (up to space or ,)
    let needle2 = format!("{}=", attr);
    if let Some(start) = s.find(&needle2) {
        let rest = &s[start + needle2.len()..];
        let end = rest.find(|c: char| c.is_whitespace() || matches!(c, ',' | ']' | ';'))
            .unwrap_or(rest.len());
        let val = rest[..end].trim_matches('"');
        if !val.is_empty() {
            return Some(val.to_string());
        }
    }
    None
}

fn shorten_zid(zid: &str) -> String {
    // Strip the first chunk before any newline (record labels use \n)
    let zid = zid.split(|c| c == '\n' || c == '\\').next().unwrap_or(zid).trim();
    if zid.len() > 8 {
        zid[..8].to_string()
    } else {
        zid.to_string()
    }
}

pub fn view(state: &AppState) -> Element<Message> {
    let fetch_row = row![
        text("Router ZID:").width(100),
        text_input("(auto-detect)", &state.graph_router)
            .on_input(Message::GraphRouterChanged),
        button("Fetch").on_press(Message::GraphFetch),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let content: Element<Message> = if let Some(ref gd) = state.graph_data {
        if gd.graph_state.nodes.is_empty() {
            // Show raw DOT so the parser can be debugged
            let raw = scrollable(
                text(gd.dot.as_str()).size(12).font(iced::Font::MONOSPACE),
            )
            .width(Length::Fill)
            .height(Length::Fill);
            column![
                text("No nodes parsed — raw DOT response:").size(13)
                    .color(iced::Color::from_rgb(1.0, 0.6, 0.2)),
                raw,
            ]
            .spacing(6)
            .into()
        } else {
            GraphCanvas::view(&gd.graph_state)
        }
    } else if let Some(ref e) = state.graph_error {
        text(e.as_str())
            .color(iced::Color::from_rgb(1.0, 0.3, 0.3))
            .into()
    } else {
        text("Press Fetch to load the routing graph.").into()
    };

    column![
        text("Graph").size(20),
        fetch_row,
        content,
    ]
    .spacing(10)
    .height(Length::Fill)
    .into()
}
