use serde_derive::*;

use crate::domain::BigNum;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorDesc {
    id: String,
    url: String,
    fee: BigNum,
}