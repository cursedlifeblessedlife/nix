use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DistTag {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct DistVersion {
    pub version: String,
}
