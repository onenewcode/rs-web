use serde::Deserialize;

#[derive(Deserialize)]
pub struct PageParams {
    pub page: Option<u64>,
    pub size: Option<u64>,
}
