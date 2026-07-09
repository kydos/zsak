use zenoh::query::{ConsolidationMode, QueryTarget};
use zenoh::qos::Priority;

#[derive(Clone, Debug)]
pub struct PublishParams {
    pub key_expr: String,
    pub value: String,
    pub attachment: Option<String>,
    pub count: u32,
    pub period_ms: u64,
    pub reliable: bool,
    pub priority: Option<Priority>,
}

impl Default for PublishParams {
    fn default() -> Self {
        Self {
            key_expr: String::new(),
            value: String::new(),
            attachment: None,
            count: 1,
            period_ms: 0,
            reliable: true,
            priority: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubscribeParams {
    pub key_expr: String,
}

#[derive(Clone, Debug)]
pub struct QueryParams {
    pub query_expr: String,
    pub body: Option<String>,
    pub attachment: Option<String>,
    pub target: QueryTarget,
    pub consolidation: ConsolidationMode,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            query_expr: String::new(),
            body: None,
            attachment: None,
            target: QueryTarget::BestMatching,
            consolidation: ConsolidationMode::None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct QueryableParams {
    pub key_expr: String,
    pub reply: String,
    pub complete: bool,
    pub exec_script: bool,
    pub packages_path: Option<String>,
}

impl Default for QueryableParams {
    fn default() -> Self {
        Self {
            key_expr: String::new(),
            reply: String::new(),
            complete: false,
            exec_script: false,
            packages_path: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScoutParams {
    pub interval_secs: u64,
}

impl Default for ScoutParams {
    fn default() -> Self {
        Self { interval_secs: 5 }
    }
}

#[derive(Clone, Debug)]
pub struct ListParams {
    pub kind: usize,
}

#[derive(Clone, Debug)]
pub struct DeleteParams {
    pub key_expr: String,
}

#[derive(Clone, Debug)]
pub struct StorageParams {
    pub key_expr: String,
    pub complete: bool,
    pub align: bool,
}

impl Default for StorageParams {
    fn default() -> Self {
        Self {
            key_expr: String::new(),
            complete: false,
            align: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GraphParams {
    pub router_zid: Option<String>,
}

impl Default for GraphParams {
    fn default() -> Self {
        Self { router_zid: None }
    }
}

#[derive(Clone, Debug)]
pub enum LivelinessAction {
    Declare(String),
    Subscribe(String),
    Query(String),
}

#[derive(Clone, Debug)]
pub struct LivelinessParams {
    pub action: LivelinessAction,
}

// Events emitted by streaming operations
#[derive(Clone, Debug)]
pub enum ZenohEvent {
    Sample {
        key: String,
        value: String,
        attachment: Option<String>,
        n: u64,
    },
    QueryIn {
        key: String,
        n: u64,
    },
    LivelinessJoin(String),
    LivelinessLeave(String),
    Error(String),
    Done,
}
