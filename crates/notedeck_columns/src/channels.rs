use std::collections::HashMap;
use enostr::Pubkey;
use nostrdb::Transaction;
use notedeck::{tr, AppContext, Localization, FALLBACK_PUBKEY};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    route::{Route, Router},
    timeline::{TimelineCache, TimelineKind},
    subscriptions::Subscriptions,
};

/// Represents a single channel (like a Slack channel)
/// Each channel filters notes by hashtag(s)
#[derive(Clone, Debug)]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
    pub hashtags: Vec<String>,
    pub timeline_kind: TimelineKind,
    pub router: Router<Route>,
    pub unread_count: usize,
    // Note: subscribed state is tracked by TimelineCache, not here
}

impl Channel {
    pub fn new(name: String, hashtags: Vec<String>) -> Self {
        let id = Uuid::new_v4();
        let timeline_kind = TimelineKind::Hashtag(hashtags.clone());
        let router = Router::new(vec![Route::timeline(timeline_kind.clone())]);

        Self {
            id,
            name,
            hashtags,
            timeline_kind,
            router,
            unread_count: 0,
        }
    }

    pub fn with_id(id: Uuid, name: String, hashtags: Vec<String>) -> Self {
        let timeline_kind = TimelineKind::Hashtag(hashtags.clone());
        let router = Router::new(vec![Route::timeline(timeline_kind.clone())]);

        Self {
            id,
            name,
            hashtags,
            timeline_kind,
            router,
            unread_count: 0,
        }
    }

    pub fn router(&self) -> &Router<Route> {
        &self.router
    }

    pub fn router_mut(&mut self) -> &mut Router<Route> {
        &mut self.router
    }
}

/// Contains all channels for a user
#[derive(Clone, Debug)]
pub struct ChannelList {
    pub channels: Vec<Channel>,
    pub selected: usize,
}

impl ChannelList {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            selected: 0,
        }
    }

    pub fn default_channels(i18n: &mut Localization) -> Self {
        let mut list = Self::new();

        // Add a default "general" channel
        list.add_channel(Channel::new(
            tr!(i18n, "General", "Default channel name").to_string(),
            vec!["general".to_string()],
        ));

        list
    }

    pub fn add_channel(&mut self, channel: Channel) {
        self.channels.push(channel);
        // Auto-select the newly added channel
        self.selected = self.channels.len() - 1;
    }

    pub fn edit_channel(
        &mut self,
        index: usize,
        name: String,
        hashtags: Vec<String>,
        timeline_cache: &mut TimelineCache,
        ndb: &mut nostrdb::Ndb,
        pool: &mut enostr::RelayPool,
    ) -> bool {
        if index >= self.channels.len() {
            return false;
        }

        let channel = &mut self.channels[index];

        // Unsubscribe from old timeline if hashtags changed
        let old_timeline_kind = channel.timeline_kind.clone();
        let new_timeline_kind = TimelineKind::Hashtag(hashtags.clone());

        if old_timeline_kind != new_timeline_kind {
            if let Err(err) = timeline_cache.pop(&old_timeline_kind, ndb, pool) {
                error!("Failed to unsubscribe from old channel timeline: {err}");
            }
        }

        // Update channel data
        channel.name = name;
        channel.hashtags = hashtags.clone();
        channel.timeline_kind = new_timeline_kind.clone();
        channel.router = Router::new(vec![Route::timeline(new_timeline_kind)]);

        info!("Updated channel: {}", channel.name);
        true
    }

    pub fn remove_channel(
        &mut self,
        index: usize,
        timeline_cache: &mut TimelineCache,
        ndb: &mut nostrdb::Ndb,
        pool: &mut enostr::RelayPool,
    ) -> Option<Channel> {
        if index < self.channels.len() && self.channels.len() > 1 {
            let removed = self.channels.remove(index);

            // Unsubscribe from the timeline
            if let Err(err) = timeline_cache.pop(&removed.timeline_kind, ndb, pool) {
                error!("Failed to unsubscribe from channel timeline: {err}");
            } else {
                info!("Unsubscribed from removed channel: {}", removed.name);
            }

            // Adjust selected index if needed
            if self.selected >= self.channels.len() {
                self.selected = self.channels.len() - 1;
            }

            Some(removed)
        } else {
            None
        }
    }

    pub fn select_channel(&mut self, index: usize) {
        if index < self.channels.len() {
            self.selected = index;
        }
    }

    pub fn selected_channel(&self) -> Option<&Channel> {
        self.channels.get(self.selected)
    }

    pub fn selected_channel_mut(&mut self) -> Option<&mut Channel> {
        self.channels.get_mut(self.selected)
    }

    pub fn num_channels(&self) -> usize {
        self.channels.len()
    }

    pub fn get_channel(&self, index: usize) -> Option<&Channel> {
        self.channels.get(index)
    }

    pub fn get_channel_mut(&mut self, index: usize) -> Option<&mut Channel> {
        self.channels.get_mut(index)
    }

    /// Subscribe to all channels' timelines
    pub fn subscribe_all(
        &mut self,
        subs: &mut Subscriptions,
        timeline_cache: &mut TimelineCache,
        ctx: &mut AppContext,
    ) {
        let txn = match Transaction::new(ctx.ndb) {
            Ok(txn) => txn,
            Err(e) => {
                error!("Failed to create transaction for channel subscription: {}", e);
                return;
            }
        };

        for channel in &mut self.channels {
            // Skip if already subscribed (check TimelineCache directly)
            if timeline_cache.get(&channel.timeline_kind).is_some() {
                continue;
            }

            if let Some(result) = timeline_cache.open(
                subs,
                ctx.ndb,
                ctx.note_cache,
                &txn,
                ctx.pool,
                &channel.timeline_kind,
            ) {
                // Process the result to handle unknown IDs and new notes
                result.process(
                    ctx.ndb,
                    ctx.note_cache,
                    &txn,
                    timeline_cache,
                    ctx.unknown_ids,
                );

                info!("Subscribed to channel: {}", channel.name);
            }
        }
    }

    /// Unsubscribe from all channels
    pub fn unsubscribe_all(
        &mut self,
        timeline_cache: &mut TimelineCache,
        ndb: &mut nostrdb::Ndb,
        pool: &mut enostr::RelayPool,
    ) {
        for channel in &mut self.channels {
            if let Err(err) = timeline_cache.pop(&channel.timeline_kind, ndb, pool) {
                error!("Failed to unsubscribe from channel timeline: {err}");
            } else {
                info!("Unsubscribed from channel: {}", channel.name);
            }
        }
    }
}

impl Default for ChannelList {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache mapping users to their channel lists
pub struct ChannelsCache {
    account_to_channels: HashMap<Pubkey, ChannelList>,
    fallback_pubkey: Pubkey,
}

impl ChannelsCache {
    pub fn new(
        mut account_to_channels: HashMap<Pubkey, ChannelList>,
        i18n: &mut Localization,
    ) -> Self {
        let fallback_pubkey = FALLBACK_PUBKEY();
        account_to_channels
            .entry(fallback_pubkey)
            .or_insert_with(|| ChannelList::default_channels(i18n));

        Self {
            account_to_channels,
            fallback_pubkey,
        }
    }

    pub fn default_channels_cache(i18n: &mut Localization) -> Self {
        let mut account_to_channels: HashMap<Pubkey, ChannelList> = Default::default();
        account_to_channels.insert(FALLBACK_PUBKEY(), ChannelList::default_channels(i18n));
        Self::new(account_to_channels, i18n)
    }

    pub fn get_channels(&self, key: &Pubkey) -> &ChannelList {
        self.account_to_channels
            .get(key)
            .unwrap_or_else(|| self.fallback())
    }

    pub fn get_channels_mut(&mut self, i18n: &mut Localization, key: &Pubkey) -> &mut ChannelList {
        self.account_to_channels
            .entry(*key)
            .or_insert_with(|| ChannelList::default_channels(i18n))
    }

    pub fn active_channels(&self, accounts: &notedeck::Accounts) -> &ChannelList {
        let account = accounts.get_selected_account();
        self.get_channels(&account.key.pubkey)
    }

    pub fn active_channels_mut(
        &mut self,
        i18n: &mut Localization,
        accounts: &notedeck::Accounts,
    ) -> &mut ChannelList {
        let account = accounts.get_selected_account();
        self.get_channels_mut(i18n, &account.key.pubkey)
    }

    pub fn selected_channel(&self, accounts: &notedeck::Accounts) -> Option<&Channel> {
        self.active_channels(accounts).selected_channel()
    }

    pub fn selected_channel_mut(
        &mut self,
        i18n: &mut Localization,
        accounts: &notedeck::Accounts,
    ) -> Option<&mut Channel> {
        self.active_channels_mut(i18n, accounts).selected_channel_mut()
    }

    pub fn fallback(&self) -> &ChannelList {
        // SAFETY: fallback_pubkey is always initialized in ChannelsCache::new()
        // and the entry is created in the constructor, so this should never fail
        self.account_to_channels
            .get(&self.fallback_pubkey)
            .expect("fallback channel list not found - this is a bug in ChannelsCache initialization")
    }

    pub fn fallback_mut(&mut self) -> &mut ChannelList {
        // SAFETY: fallback_pubkey is always initialized in ChannelsCache::new()
        // and the entry is created in the constructor, so this should never fail
        self.account_to_channels
            .get_mut(&self.fallback_pubkey)
            .expect("fallback channel list not found - this is a bug in ChannelsCache initialization")
    }

    pub fn add_channel_for_account(
        &mut self,
        i18n: &mut Localization,
        pubkey: Pubkey,
        channel: Channel,
    ) {
        let channel_name = channel.name.clone();
        let channels = self.get_channels_mut(i18n, &pubkey);
        channels.add_channel(channel);
        info!("Added channel '{}' for {:?}", channel_name, pubkey);
    }

    pub fn remove(
        &mut self,
        i18n: &mut Localization,
        key: &Pubkey,
        timeline_cache: &mut TimelineCache,
        ndb: &mut nostrdb::Ndb,
        pool: &mut enostr::RelayPool,
    ) {
        let Some(mut channels) = self.account_to_channels.remove(key) else {
            return;
        };
        info!("Removing channels for {:?}", key);

        channels.unsubscribe_all(timeline_cache, ndb, pool);

        if !self.account_to_channels.contains_key(&self.fallback_pubkey) {
            self.account_to_channels
                .insert(self.fallback_pubkey, ChannelList::default_channels(i18n));
        }
    }

    pub fn get_fallback_pubkey(&self) -> &Pubkey {
        &self.fallback_pubkey
    }

    pub fn get_mapping(&self) -> &HashMap<Pubkey, ChannelList> {
        &self.account_to_channels
    }
}
