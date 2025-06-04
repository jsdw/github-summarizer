use std::str::FromStr;
use std::fmt;
use time::format_description::well_known::Iso8601;

/// A DateTime that can be parsed from/to ISO 8601 repr.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime(pub time::OffsetDateTime);

impl DateTime {
    pub fn now() -> Self {
        DateTime(time::OffsetDateTime::now_utc())
    }
}

impl serde::ser::Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.0.format(&Iso8601::DEFAULT).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&s)
    }
}

impl <'de> serde::Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for DateTime {
    type Err = time::error::Parse;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dt = time::OffsetDateTime::parse(s, &Iso8601::DEFAULT)?;
        Ok(DateTime(dt))
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0.format(&Iso8601::DEFAULT)
            .expect("Failed to format time");
        write!(f, "{s}")
    }
}

/// Items like PRs and issues have a state in the girhub API. This enum can represent that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemState {
    Open,
    Closed,
    Merged
}

impl FromStr for ItemState {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OPEN" => Ok(ItemState::Open),
            "CLOSED" => Ok(ItemState::Closed),
            "MERGED" => Ok(ItemState::Merged),
            _ => Err(anyhow::anyhow!("Unknown ItemState variant: {s:?}, expecting OPEN, CLOSED or MERGED")),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ItemState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ItemState::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl serde::ser::Serialize for ItemState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            ItemState::Open => "OPEN",
            ItemState::Closed => "CLOSED",
            ItemState::Merged => "MERGED",
        };
        serializer.serialize_str(s)
    }
}