use typeshare::typeshare;

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct IntegrationDiagnosticEventTs {
    pub timestamp: String,
    pub store: String,
    pub stage: String,
    pub trigger: String,
    pub page_kind: String,
    pub count: Option<u32>,
    pub detail: String,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct RecordDiagnosticsRequestTs {
    pub events: Vec<IntegrationDiagnosticEventTs>,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct RecordDiagnosticsResultTs {
    pub recorded: u32,
}
