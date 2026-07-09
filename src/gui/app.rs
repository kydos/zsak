use std::sync::Arc;
use tokio::sync::oneshot;

use iced::widget::{button, column, container, horizontal_rule, row, text, text_input};
use iced::{Element, Length, Task, Theme};

use crate::session::{build_config, SessionConfig};
use crate::views;
use zsak::types::ZenohEvent;

// ---------------------------------------------------------------------------
// Navigation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    Doctor,
    Scout,
    List,
    Publish,
    Delete,
    Subscribe,
    Query,
    Queryable,
    Storage,
    Liveliness,
    Graph,
}

// ---------------------------------------------------------------------------
// Application-level messages
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    Navigate(View),

    // Session
    ShowConfig,
    HideConfig,
    ConfigFileChanged(String),
    ConfigModeChanged(String),
    ConfigEndpointsChanged(String),
    ConfigListenChanged(String),
    ConfigNameChanged(String),
    ConfigAdminToggled(bool),
    ConfigRestPortChanged(String),
    ConfigMulticastToggled(bool),
    ConnectSession,
    SessionConnected(Arc<zenoh::Session>),
    SessionError(String),

    // Streaming lifecycle
    StreamEvent(ZenohEvent),
    StopStream,

    // View-specific
    DoctorRun,
    DoctorResult(String),
    ScoutRun,
    ScoutResult(Vec<(String, String)>),
    ListRun,
    ListResult(Vec<(String, String)>),
    PublishKeyChanged(String),
    PublishValueChanged(String),
    PublishCountChanged(String),
    PublishPeriodChanged(String),
    PublishRun,
    PublishResult(Vec<String>),
    DeleteKeyChanged(String),
    DeleteRun,
    DeleteResult(String),
    SubscribeKeyChanged(String),
    SubscribeStart,
    SubscribeStop,
    QueryExprChanged(String),
    QueryBodyChanged(String),
    QueryRun,
    QueryResult(Vec<String>),
    QueryableKeyChanged(String),
    QueryableReplyChanged(String),
    QueryableStart,
    QueryableStop,
    StorageKeyChanged(String),
    StorageStart,
    StorageStop,
    LivelinessKeyChanged(String),
    LivelinessActionChanged(String),
    LivelinessRun,
    LivelinessStart,
    LivelinessStop,
    GraphRouterChanged(String),
    GraphFetch,
    GraphResult(Result<crate::views::graph::GraphData, String>),
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

pub struct AppState {
    pub current_view: View,
    pub show_config: bool,
    pub session_config: SessionConfig,
    pub session: Option<Arc<zenoh::Session>>,
    pub session_error: Option<String>,
    pub cancel_tx: Option<oneshot::Sender<()>>,

    // Per-view state
    pub log_lines: Vec<String>,
    pub publish_key: String,
    pub publish_value: String,
    pub publish_count: String,
    pub publish_period: String,
    pub delete_key: String,
    pub subscribe_key: String,
    pub subscribe_active: bool,
    pub query_expr: String,
    pub query_body: String,
    pub queryable_key: String,
    pub queryable_reply: String,
    pub queryable_active: bool,
    pub storage_key: String,
    pub storage_active: bool,
    pub storage_process: Option<tokio::process::Child>,
    pub liveliness_key: String,
    pub liveliness_action: String, // "declare" | "subscribe" | "query"
    pub liveliness_active: bool,
    pub graph_router: String,
    pub graph_data: Option<crate::views::graph::GraphData>,
    pub graph_error: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_view: View::Doctor,
            show_config: false,
            session_config: SessionConfig::default(),
            session: None,
            session_error: None,
            cancel_tx: None,
            log_lines: Vec::new(),
            publish_key: "demo/key".into(),
            publish_value: "hello".into(),
            publish_count: "1".into(),
            publish_period: "0".into(),
            delete_key: String::new(),
            subscribe_key: "demo/**".into(),
            subscribe_active: false,
            query_expr: "demo/**".into(),
            query_body: String::new(),
            queryable_key: "demo/queryable".into(),
            queryable_reply: "pong".into(),
            queryable_active: false,
            storage_key: "demo/**".into(),
            storage_active: false,
            storage_process: None,
            liveliness_key: "demo/**".into(),
            liveliness_action: "query".into(),
            liveliness_active: false,
            graph_router: String::new(),
            graph_data: None,
            graph_error: None,
        }
    }
}

// ---------------------------------------------------------------------------
// iced Application impl
// ---------------------------------------------------------------------------

impl AppState {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn title(&self) -> String {
        "zsak — Zenoh Swiss Army Knife".into()
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(v) => {
                self.stop_stream();
                self.current_view = v;
                self.log_lines.clear();
                Task::none()
            }

            Message::ShowConfig => { self.show_config = true; Task::none() }
            Message::HideConfig => { self.show_config = false; Task::none() }
            Message::ConfigFileChanged(v) => {
                self.session_config.config_file = if v.is_empty() { None } else { Some(v) };
                Task::none()
            }
            Message::ConfigModeChanged(v) => { self.session_config.mode = v; Task::none() }
            Message::ConfigEndpointsChanged(v) => { self.session_config.endpoints = v; Task::none() }
            Message::ConfigListenChanged(v) => { self.session_config.listen = v; Task::none() }
            Message::ConfigNameChanged(v) => { self.session_config.name = v; Task::none() }
            Message::ConfigAdminToggled(v) => { self.session_config.admin_enabled = v; Task::none() }
            Message::ConfigRestPortChanged(v) => { self.session_config.rest_port = v; Task::none() }
            Message::ConfigMulticastToggled(v) => { self.session_config.multicast_scouting = v; Task::none() }

            Message::ConnectSession => {
                self.session_error = None;
                let config = build_config(&self.session_config);
                Task::future(async move {
                    match zenoh::open(config).await {
                        Ok(s) => Message::SessionConnected(Arc::new(s)),
                        Err(e) => Message::SessionError(e.to_string()),
                    }
                })
            }
            Message::SessionConnected(s) => {
                self.session = Some(s);
                self.show_config = false;
                Task::none()
            }
            Message::SessionError(e) => { self.session_error = Some(e); Task::none() }

            Message::StreamEvent(ev) => {
                use ZenohEvent::*;
                match ev {
                    Sample { key, value, attachment, n } => {
                        if let Some(a) = attachment {
                            self.log_lines.push(format!("[{}] {}: {} (attach: {})", n, key, value, a));
                        } else {
                            self.log_lines.push(format!("[{}] {}: {}", n, key, value));
                        }
                    }
                    QueryIn { key, n } => {
                        self.log_lines.push(format!("[{}] Query: {}", n, key));
                    }
                    LivelinessJoin(k) => self.log_lines.push(format!("JOIN  {}", k)),
                    LivelinessLeave(k) => self.log_lines.push(format!("LEAVE {}", k)),
                    Error(e) => self.log_lines.push(format!("ERROR: {}", e)),
                    Done => self.log_lines.push("Done.".into()),
                }
                Task::none()
            }
            Message::StopStream => { self.stop_stream(); Task::none() }

            // Doctor
            Message::DoctorRun => {
                Task::future(async {
                    let zsak_home = std::env::var("ZSAK_HOME")
                        .map(|_| "ZSAK_HOME: OK".to_string())
                        .unwrap_or_else(|_| "ZSAK_HOME: NOT SET".to_string());
                    let zenohd = if tokio::process::Command::new("zenohd")
                        .arg("-h").output().await.is_ok()
                    {
                        "zenohd: found in PATH"
                    } else {
                        "zenohd: NOT found in PATH"
                    };
                    Message::DoctorResult(format!("{}\n{}", zsak_home, zenohd))
                })
            }
            Message::DoctorResult(s) => {
                self.log_lines.clear();
                for line in s.lines() { self.log_lines.push(line.to_string()); }
                Task::none()
            }

            // Scout
            Message::ScoutRun => {
                if let Some(ref z) = self.session {
                    let z = z.clone();
                    Task::future(async move {
                        let result = zsak::action::do_scout(&z, 5).await;
                        let items = result.values()
                            .map(|h| (h.zid().to_string(), h.whatami().to_string()))
                            .collect();
                        Message::ScoutResult(items)
                    })
                } else {
                    self.log_lines.push("Not connected. Open Config and Connect first.".into());
                    Task::none()
                }
            }
            Message::ScoutResult(items) => {
                self.log_lines.clear();
                if items.is_empty() { self.log_lines.push("No runtimes found.".into()); }
                for (zid, kind) in items { self.log_lines.push(format!("{} ({})", zid, kind)); }
                Task::none()
            }

            // List
            Message::ListRun => {
                if let Some(ref z) = self.session {
                    let z = z.clone();
                    Task::future(async move {
                        use zenoh::config::WhatAmI;
                        let kind = WhatAmI::Router as usize | WhatAmI::Peer as usize | WhatAmI::Client as usize;
                        let result = zsak::action::do_list(&z, kind).await;
                        let items = result.into_iter().map(|(id, wai)| (id, wai.to_string())).collect();
                        Message::ListResult(items)
                    })
                } else {
                    self.log_lines.push("Not connected.".into());
                    Task::none()
                }
            }
            Message::ListResult(items) => {
                self.log_lines.clear();
                if items.is_empty() { self.log_lines.push("No nodes found.".into()); }
                for (id, kind) in items { self.log_lines.push(format!("{} ({})", id, kind)); }
                Task::none()
            }

            // Publish
            Message::PublishKeyChanged(v) => { self.publish_key = v; Task::none() }
            Message::PublishValueChanged(v) => { self.publish_value = v; Task::none() }
            Message::PublishCountChanged(v) => { self.publish_count = v; Task::none() }
            Message::PublishPeriodChanged(v) => { self.publish_period = v; Task::none() }
            Message::PublishRun => {
                if let Some(ref z) = self.session {
                    let z = z.clone();
                    let p = zsak::types::PublishParams {
                        key_expr: self.publish_key.clone(),
                        value: self.publish_value.clone(),
                        attachment: None,
                        count: self.publish_count.parse().unwrap_or(1),
                        period_ms: self.publish_period.parse().unwrap_or(0),
                        reliable: true,
                        priority: None,
                    };
                    self.log_lines.clear();
                    Task::future(async move {
                        let events = zsak::action::do_publish_with(&z, &p).await;
                        let lines = events.into_iter().map(|e| match e {
                            zsak::types::ZenohEvent::Sample { key, value, n, .. } =>
                                format!("[{}] {}: {}", n, key, value),
                            _ => format!("{:?}", e),
                        }).collect();
                        Message::PublishResult(lines)
                    })
                } else {
                    self.log_lines.push("Not connected.".into());
                    Task::none()
                }
            }
            Message::PublishResult(lines) => {
                self.log_lines.extend(lines);
                Task::none()
            }

            // Delete
            Message::DeleteKeyChanged(v) => { self.delete_key = v; Task::none() }
            Message::DeleteRun => {
                if let Some(ref z) = self.session {
                    let z = z.clone();
                    let key = self.delete_key.clone();
                    Task::future(async move {
                        zsak::action::do_delete_with(&z, &zsak::types::DeleteParams { key_expr: key.clone() }).await;
                        Message::DeleteResult(format!("Deleted: {}", key))
                    })
                } else {
                    self.log_lines.push("Not connected.".into());
                    Task::none()
                }
            }
            Message::DeleteResult(s) => { self.log_lines.push(s); Task::none() }

            // Subscribe
            Message::SubscribeKeyChanged(v) => { self.subscribe_key = v; Task::none() }
            Message::SubscribeStart => {
                if let Some(ref z) = self.session {
                    let (task, cancel) = crate::bridge::subscribe_stream(
                        z.clone(),
                        zsak::types::SubscribeParams { key_expr: self.subscribe_key.clone() },
                    );
                    self.cancel_tx = Some(cancel);
                    self.subscribe_active = true;
                    self.log_lines.clear();
                    task
                } else {
                    self.log_lines.push("Not connected.".into());
                    Task::none()
                }
            }
            Message::SubscribeStop => { self.stop_stream(); self.subscribe_active = false; Task::none() }

            // Query
            Message::QueryExprChanged(v) => { self.query_expr = v; Task::none() }
            Message::QueryBodyChanged(v) => { self.query_body = v; Task::none() }
            Message::QueryRun => {
                if let Some(ref z) = self.session {
                    let z = z.clone();
                    let p = zsak::types::QueryParams {
                        query_expr: self.query_expr.clone(),
                        body: if self.query_body.is_empty() { None } else { Some(self.query_body.clone()) },
                        ..Default::default()
                    };
                    Task::future(async move {
                        let events = zsak::action::do_query_with(&z, &p).await;
                        let lines = events.iter().map(|e| format!("{:?}", e)).collect();
                        Message::QueryResult(lines)
                    })
                } else {
                    self.log_lines.push("Not connected.".into());
                    Task::none()
                }
            }
            Message::QueryResult(lines) => {
                self.log_lines.clear();
                self.log_lines.extend(lines);
                Task::none()
            }

            // Queryable
            Message::QueryableKeyChanged(v) => { self.queryable_key = v; Task::none() }
            Message::QueryableReplyChanged(v) => { self.queryable_reply = v; Task::none() }
            Message::QueryableStart => {
                if let Some(ref z) = self.session {
                    let (task, cancel) = crate::bridge::queryable_stream(
                        z.clone(),
                        zsak::types::QueryableParams {
                            key_expr: self.queryable_key.clone(),
                            reply: self.queryable_reply.clone(),
                            ..Default::default()
                        },
                    );
                    self.cancel_tx = Some(cancel);
                    self.queryable_active = true;
                    self.log_lines.clear();
                    task
                } else {
                    self.log_lines.push("Not connected.".into());
                    Task::none()
                }
            }
            Message::QueryableStop => { self.stop_stream(); self.queryable_active = false; Task::none() }

            // Storage
            Message::StorageKeyChanged(v) => { self.storage_key = v; Task::none() }
            Message::StorageStart => {
                let storage_cfg = format!(
                    "{{ key_expr: \"{}\", volume: \"memory\", complete: \"false\" }}",
                    self.storage_key
                );
                if let Some(path) = std::env::var_os("ZSAK_HOME") {
                    let config_path = path.into_string().unwrap() + "/config/config.json5";
                    self.storage_active = true;
                    self.log_lines.push(format!("Starting storage for: {}", self.storage_key));
                    let rt = tokio::runtime::Handle::current();
                    let child = rt.block_on(async {
                        let cfg_template = tokio::fs::read_to_string(&config_path).await.unwrap_or_default();
                        let cfg = cfg_template.replace("$STORAGE", &storage_cfg);
                        tokio::process::Command::new("zenohd").args(["--cfg", &cfg]).spawn().ok()
                    });
                    self.storage_process = child;
                } else {
                    self.log_lines.push("ZSAK_HOME not set — cannot start storage.".into());
                }
                Task::none()
            }
            Message::StorageStop => {
                if let Some(mut child) = self.storage_process.take() { let _ = child.start_kill(); }
                self.storage_active = false;
                self.log_lines.push("Storage stopped.".into());
                Task::none()
            }

            // Liveliness
            Message::LivelinessKeyChanged(v) => { self.liveliness_key = v; Task::none() }
            Message::LivelinessActionChanged(v) => { self.liveliness_action = v; Task::none() }
            Message::LivelinessRun => {
                if let Some(ref z) = self.session {
                    let z = z.clone();
                    let key = self.liveliness_key.clone();
                    Task::future(async move {
                        let p = zsak::types::LivelinessParams {
                            action: zsak::types::LivelinessAction::Query(key),
                        };
                        let events = zsak::action::do_liveliness_with(&z, &p).await;
                        let lines = events.iter().map(|e| format!("{:?}", e)).collect();
                        Message::QueryResult(lines)
                    })
                } else {
                    self.log_lines.push("Not connected.".into());
                    Task::none()
                }
            }
            Message::LivelinessStart => {
                if let Some(ref z) = self.session {
                    let key = self.liveliness_key.clone();
                    let action = self.liveliness_action.clone();
                    if action == "subscribe" {
                        let (task, cancel) = crate::bridge::liveliness_subscribe_stream(z.clone(), key);
                        self.cancel_tx = Some(cancel);
                        self.liveliness_active = true;
                        self.log_lines.clear();
                        task
                    } else {
                        self.liveliness_active = true;
                        self.log_lines.push(format!("Declared liveliness token: {}", key));
                        // Note: token lifetime is not managed here — dropped immediately.
                        // A production impl would store the token in AppState.
                        Task::future(async move {
                            Message::StreamEvent(ZenohEvent::Done)
                        })
                    }
                } else {
                    self.log_lines.push("Not connected.".into());
                    Task::none()
                }
            }
            Message::LivelinessStop => { self.stop_stream(); self.liveliness_active = false; Task::none() }

            // Graph
            Message::GraphRouterChanged(v) => { self.graph_router = v; Task::none() }
            Message::GraphFetch => {
                if let Some(ref z) = self.session {
                    let z = z.clone();
                    let router_zid = if self.graph_router.is_empty() { None } else { Some(self.graph_router.clone()) };
                    self.graph_data = None;
                    self.graph_error = None;
                    Task::future(async move {
                        let p = zsak::types::GraphParams { router_zid };
                        match zsak::action::do_graph_with(&z, &p).await {
                            Ok(dot) => {
                                let gd = crate::views::graph::parse_dot(&dot);
                                Message::GraphResult(Ok(gd))
                            }
                            Err(e) => Message::GraphResult(Err(e)),
                        }
                    })
                } else {
                    self.graph_error = Some("Not connected.".into());
                    Task::none()
                }
            }
            Message::GraphResult(r) => {
                match r {
                    Ok(gd) => { self.graph_data = Some(gd); self.graph_error = None; }
                    Err(e) => { self.graph_error = Some(e); }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let sidebar = views::sidebar::sidebar_view(&self.current_view, self.session.is_some());
        let content = self.active_view();

        let header = row![
            text("zsak — Zenoh Swiss Army Knife").size(16),
            iced::widget::horizontal_space(),
            button("Config...").on_press(Message::ShowConfig),
        ]
        .padding(8)
        .spacing(8)
        .align_y(iced::Alignment::Center);

        let body = row![
            container(sidebar).width(160).height(Length::Fill),
            iced::widget::vertical_rule(1),
            container(content).width(Length::Fill).height(Length::Fill).padding(12),
        ]
        .height(Length::Fill);

        let main_col = column![
            header,
            horizontal_rule(1),
            body,
        ];

        if self.show_config {
            let overlay = views::config_modal(self);
            iced::widget::stack![main_col, overlay].into()
        } else {
            main_col.into()
        }
    }

    fn active_view(&self) -> Element<Message> {
        match self.current_view {
            View::Doctor => views::doctor::view(self),
            View::Scout => views::scout::view(self),
            View::List => views::list::view(self),
            View::Publish => views::publish::view(self),
            View::Delete => views::delete::view(self),
            View::Subscribe => views::subscribe::view(self),
            View::Query => views::query::view(self),
            View::Queryable => views::queryable::view(self),
            View::Storage => views::storage::view(self),
            View::Liveliness => views::liveliness::view(self),
            View::Graph => views::graph::view(self),
        }
    }

    pub fn stop_stream(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(());
        }
    }
}
