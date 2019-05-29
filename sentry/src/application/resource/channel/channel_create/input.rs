use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tower_web::Extract;

use crate::domain::{Asset, BigNum, ChannelId, ChannelSpec, Identifier};

#[derive(Extract, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChannelInput {
    pub id: ChannelId,
    pub creator: Identifier,
    pub deposit_asset: Asset,
    pub deposit_amount: BigNum,
    pub valid_until: DateTime<Utc>,
    pub spec: ChannelSpec,
}