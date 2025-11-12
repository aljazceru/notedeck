use std::collections::HashMap;

use enostr::Pubkey;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use uuid::Uuid;

use crate::channels::{Channel, ChannelList, ChannelsCache};

use notedeck::{storage, DataPath, DataPathType, Directory, Localization};

pub static CHANNELS_CACHE_FILE: &str = "channels_cache.json";

pub fn load_channels_cache(path: &DataPath, i18n: &mut Localization) -> Option<ChannelsCache> {
    let data_path = path.path(DataPathType::Setting);

    let channels_cache_str = match Directory::new(data_path).get_file(CHANNELS_CACHE_FILE.to_owned()) {
        Ok(s) => s,
        Err(e) => {
            error!(
                "Could not read channels cache from file {}: {}",
                CHANNELS_CACHE_FILE, e
            );
            return None;
        }
    };

    let serializable_channels_cache =
        serde_json::from_str::<SerializableChannelsCache>(&channels_cache_str).ok()?;

    Some(serializable_channels_cache.channels_cache(i18n))
}

pub fn save_channels_cache(path: &DataPath, channels_cache: &ChannelsCache) {
    let serialized_channels_cache =
        match serde_json::to_string_pretty(&SerializableChannelsCache::to_serializable(channels_cache)) {
            Ok(s) => s,
            Err(e) => {
                error!("Could not serialize channels cache: {}", e);
                return;
            }
        };

    let data_path = path.path(DataPathType::Setting);

    if let Err(e) = storage::write_file(
        &data_path,
        CHANNELS_CACHE_FILE.to_string(),
        &serialized_channels_cache,
    ) {
        error!(
            "Could not write channels cache to file {}: {}",
            CHANNELS_CACHE_FILE, e
        );
    } else {
        debug!("Successfully wrote channels cache to {}", CHANNELS_CACHE_FILE);
    }
}

#[derive(Serialize, Deserialize)]
struct SerializableChannelsCache {
    #[serde(serialize_with = "serialize_map", deserialize_with = "deserialize_map")]
    channels_cache: HashMap<Pubkey, SerializableChannelList>,
}

impl SerializableChannelsCache {
    fn to_serializable(channels_cache: &ChannelsCache) -> Self {
        SerializableChannelsCache {
            channels_cache: channels_cache
                .get_mapping()
                .iter()
                .map(|(k, v)| (*k, SerializableChannelList::from_channel_list(v)))
                .collect(),
        }
    }

    pub fn channels_cache(self, i18n: &mut Localization) -> ChannelsCache {
        let account_to_channels = self
            .channels_cache
            .into_iter()
            .map(|(pubkey, serializable_channels)| {
                (pubkey, serializable_channels.channel_list())
            })
            .collect();

        ChannelsCache::new(account_to_channels, i18n)
    }
}

fn serialize_map<S>(
    map: &HashMap<Pubkey, SerializableChannelList>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let stringified_map: HashMap<String, &SerializableChannelList> =
        map.iter().map(|(k, v)| (k.hex(), v)).collect();
    stringified_map.serialize(serializer)
}

fn deserialize_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<Pubkey, SerializableChannelList>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let stringified_map: HashMap<String, SerializableChannelList> =
        HashMap::deserialize(deserializer)?;

    stringified_map
        .into_iter()
        .map(|(k, v)| {
            let key = Pubkey::from_hex(&k).map_err(serde::de::Error::custom)?;
            Ok((key, v))
        })
        .collect()
}

#[derive(Serialize, Deserialize)]
struct SerializableChannelList {
    channels: Vec<SerializableChannel>,
    selected: usize,
}

impl SerializableChannelList {
    pub fn from_channel_list(channel_list: &ChannelList) -> Self {
        Self {
            channels: channel_list
                .channels
                .iter()
                .map(SerializableChannel::from_channel)
                .collect(),
            selected: channel_list.selected,
        }
    }

    fn channel_list(self) -> ChannelList {
        ChannelList {
            channels: self
                .channels
                .into_iter()
                .map(|c| c.channel())
                .collect(),
            selected: self.selected,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SerializableChannel {
    id: String,
    name: String,
    hashtags: Vec<String>,
}

impl SerializableChannel {
    pub fn from_channel(channel: &Channel) -> Self {
        Self {
            id: channel.id.to_string(),
            name: channel.name.clone(),
            hashtags: channel.hashtags.clone(),
        }
    }

    pub fn channel(self) -> Channel {
        let id = Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::new_v4());
        Channel::with_id(id, self.name, self.hashtags)
    }
}
