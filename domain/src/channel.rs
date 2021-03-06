use std::convert::TryFrom;
use std::fmt;

use chrono::serde::{ts_milliseconds, ts_seconds};
use chrono::{DateTime, Utc};
use hex::FromHex;
use serde::{Deserialize, Serialize};
use serde_hex::{SerHex, StrictPfx};

use crate::big_num::BigNum;
use crate::util::serde::ts_milliseconds_option;
use crate::{
    AdUnit, Asset, DomainError, EventSubmission, TargetingTag, ValidatorDesc, ValidatorId,
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
#[serde(transparent)]
pub struct ChannelId {
    #[serde(with = "SerHex::<StrictPfx>")]
    pub bytes: [u8; 32],
}

impl fmt::Display for ChannelId {
    /// Converts a ChannelId to hex string with prefix
    ///
    /// Example:
    /// ```
    /// use domain::ChannelId;
    ///
    /// let hex_string = "0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088";
    /// let channel_id = ChannelId::try_from_hex(&hex_string).expect("Should be a valid hex string already");
    /// let channel_hex_string = format!("{}", channel_id);
    /// assert_eq!("0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088", &channel_hex_string);
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_string = SerHex::<StrictPfx>::into_hex(&self.bytes).unwrap();
        write!(f, "{}", hex_string)
    }
}

impl TryFrom<&str> for ChannelId {
    type Error = DomainError;

    /// Tries to create a ChannelId from &str, which should be 32 bytes length.
    ///
    /// Example:
    ///
    /// ```
    /// use std::convert::TryFrom;
    /// use domain::channel::ChannelId;
    ///
    /// let bytes: [u8; 32] = *b"12345678901234567890123456789012";
    ///
    /// assert_eq!(ChannelId { bytes }, ChannelId::try_from("12345678901234567890123456789012").unwrap())
    /// ```
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();
        if bytes.len() != 32 {
            return Err(DomainError::InvalidArgument(
                "The value of the id should have exactly 32 bytes".to_string(),
            ));
        }
        let mut id = [0; 32];
        id.copy_from_slice(&bytes[..32]);

        Ok(Self { bytes: id })
    }
}

impl ChannelId {
    /// Creates a ChannelId from a string hex with or without `0x` prefix.
    /// The bytes should be 32 after decoding.
    ///
    /// Example:
    ///
    /// ```
    /// use domain::ChannelId;
    ///
    /// let hex_string = "0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088";
    ///
    /// let from_hex = domain::ChannelId::try_from_hex(hex_string).expect("This should be valid hex string");
    ///
    /// let expected_channel_id = ChannelId{ bytes: [6, 29, 94, 42, 103, 208, 169, 161, 15, 28, 115, 43, 202, 18, 166, 118, 216, 63, 121, 102, 58, 57, 111, 125, 135, 179, 227, 11, 155, 65, 16, 136]};
    /// assert_eq!(expected_channel_id, from_hex)
    /// ```
    pub fn try_from_hex(hex: &str) -> Result<Self, DomainError> {
        let s = hex.trim_start_matches("0x");

        let bytes: Vec<u8> =
            Vec::from_hex(s).map_err(|err| DomainError::InvalidArgument(err.to_string()))?;
        if bytes.len() != 32 {
            return Err(DomainError::InvalidArgument(
                "The value of the id should have exactly 32 bytes".to_string(),
            ));
        }

        let mut id = [0; 32];
        id.copy_from_slice(&bytes[..32]);

        Ok(Self { bytes: id })
    }
}

impl PartialEq<ChannelId> for &str {
    fn eq(&self, channel_id: &ChannelId) -> bool {
        self.as_bytes() == channel_id.bytes
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub id: ChannelId,
    pub creator: String,
    pub deposit_asset: Asset,
    pub deposit_amount: BigNum,
    #[serde(with = "ts_seconds")]
    pub valid_until: DateTime<Utc>,
    pub spec: ChannelSpec,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub validators: SpecValidators,
    /// Maximum payment per impression
    pub max_per_impression: BigNum,
    /// Minimum payment offered per impression
    pub min_per_impression: BigNum,
    /// An array of TargetingTag (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targeting: Vec<TargetingTag>,
    /// Minimum targeting score (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_targeting_score: Option<u64>,
    /// EventSubmission object, applies to event submission (POST /channel/:id/events)
    pub event_submission: EventSubmission,
    /// A millisecond timestamp of when the campaign was created
    #[serde(with = "ts_milliseconds")]
    pub created: DateTime<Utc>,
    /// A millisecond timestamp representing the time you want this campaign to become active (optional)
    /// Used by the AdViewManager
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "ts_milliseconds_option"
    )]
    pub active_from: Option<DateTime<Utc>>,
    /// A random number to ensure the campaignSpec hash is unique
    pub nonce: BigNum,
    /// A millisecond timestamp of when the campaign should enter a withdraw period
    /// (no longer accept any events other than CHANNEL_CLOSE)
    /// A sane value should be lower than channel.validUntil * 1000 and higher than created
    /// It's recommended to set this at least one month prior to channel.validUntil * 1000
    #[serde(with = "ts_milliseconds")]
    pub withdraw_period_start: DateTime<Utc>,
    /// An array of AdUnit (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ad_units: Vec<AdUnit>,
}

pub enum SpecValidator<'a> {
    Leader(&'a ValidatorDesc),
    Follower(&'a ValidatorDesc),
    None,
}

impl<'a> SpecValidator<'a> {
    pub fn is_some(&self) -> bool {
        match &self {
            SpecValidator::None => false,
            _ => true,
        }
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct SpecValidators([ValidatorDesc; 2]);

impl SpecValidators {
    pub fn new(leader: ValidatorDesc, follower: ValidatorDesc) -> Self {
        Self([leader, follower])
    }

    pub fn leader(&self) -> &ValidatorDesc {
        &self.0[0]
    }

    pub fn follower(&self) -> &ValidatorDesc {
        &self.0[1]
    }

    pub fn find(&self, validator: &ValidatorId) -> SpecValidator<'_> {
        if &self.leader().id == validator {
            SpecValidator::Leader(&self.leader())
        } else if &self.follower().id == validator {
            SpecValidator::Follower(&self.follower())
        } else {
            SpecValidator::None
        }
    }
}

impl From<[ValidatorDesc; 2]> for SpecValidators {
    fn from(slice: [ValidatorDesc; 2]) -> Self {
        Self(slice)
    }
}

impl<'a> IntoIterator for &'a SpecValidators {
    type Item = &'a ValidatorDesc;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.leader(), self.follower()].into_iter()
    }
}

#[cfg(any(test, feature = "fixtures"))]
#[path = "./channel_fixtures.rs"]
pub mod fixtures;

#[cfg(test)]
#[path = "./channel_test.rs"]
mod test;
