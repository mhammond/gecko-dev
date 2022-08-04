/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::error::*;
use crate::storage::{ClientRemoteTabs, RemoteTab};
use crate::sync::record::{TabsRecord, TabsRecordTab};
use crate::TabsStore;
use std::collections::HashMap;
use std::sync::Arc;
use sync15_traits::{
    client::{ClientData, RemoteClient},
    DeviceType, EngineSyncAssociation, ServerTimestamp,
};
use sync_guid::Guid;

const TTL_1_YEAR: u32 = 31_622_400;

impl ClientRemoteTabs {
    fn from_record_with_remote_client(
        client_id: String,
        remote_client: &RemoteClient,
        record: TabsRecord,
    ) -> Self {
        Self {
            client_id,
            client_name: remote_client.device_name.clone(),
            device_type: remote_client.device_type.unwrap_or(DeviceType::Unknown),
            remote_tabs: record.tabs.iter().map(RemoteTab::from_record_tab).collect(),
        }
    }

    fn from_record(client_id: String, record: TabsRecord) -> Self {
        Self {
            client_id,
            client_name: record.client_name,
            device_type: DeviceType::Unknown,
            remote_tabs: record.tabs.iter().map(RemoteTab::from_record_tab).collect(),
        }
    }
    fn to_record(&self) -> TabsRecord {
        TabsRecord {
            id: self.client_id.clone(),
            client_name: self.client_name.clone(),
            tabs: self
                .remote_tabs
                .iter()
                .map(RemoteTab::to_record_tab)
                .collect(),
            ttl: TTL_1_YEAR,
        }
    }
}

impl RemoteTab {
    fn from_record_tab(tab: &TabsRecordTab) -> Self {
        Self {
            title: tab.title.clone(),
            url_history: tab.url_history.clone(),
            icon: tab.icon.clone(),
            last_used: tab.last_used.checked_mul(1000).unwrap_or_default(),
        }
    }
    pub(super) fn to_record_tab(&self) -> TabsRecordTab {
        TabsRecordTab {
            title: self.title.clone(),
            url_history: self.url_history.clone(),
            icon: self.icon.clone(),
            last_used: self.last_used.checked_div(1000).unwrap_or_default(),
        }
    }
}

// This is the core implementation of a sync engine.
// It is described purely in sync15-traits.
pub struct TabsSyncImpl {
    pub(super) store: Arc<TabsStore>,
    remote_clients: HashMap<String, RemoteClient>,
    pub(super) last_sync: Option<ServerTimestamp>,
    sync_store_assoc: EngineSyncAssociation,
    pub(super) local_id: String,
}

impl TabsSyncImpl {
    pub fn new(store: Arc<TabsStore>) -> Self {
        Self {
            store,
            remote_clients: HashMap::new(),
            last_sync: None,
            sync_store_assoc: EngineSyncAssociation::Disconnected,
            local_id: Default::default(),
        }
    }

    pub fn prepare_for_sync(&mut self, client_data: ClientData) -> Result<()> {
        self.remote_clients = client_data.recent_clients;
        self.local_id = client_data.local_client_id;
        Ok(())
    }

    pub fn apply_incoming(&mut self, inbound: Vec<TabsRecord>) -> Result<Option<TabsRecord>> {
        let local_id = self.local_id.clone();
        let mut remote_tabs = Vec::with_capacity(inbound.len());

        for record in inbound {
            if record.id == local_id {
                // That's our own record, ignore it.
                continue;
            }
            let id = record.id.clone();
            let crt = if let Some(remote_client) = self.remote_clients.get(&id) {
                ClientRemoteTabs::from_record_with_remote_client(
                    remote_client
                        .fxa_device_id
                        .as_ref()
                        .unwrap_or(&id)
                        .to_owned(),
                    remote_client,
                    record,
                )
            } else {
                // A record with a device that's not in our remote clients seems unlikely, but
                // could happen - in most cases though, it will be due to a disconnected client -
                // so we really should consider just dropping it? (Sadly though, it does seem
                // possible it's actually a very recently connected client, so we keep it)
                log::info!(
                    "Storing tabs from a client that doesn't appear in the devices list: {}",
                    id,
                );
                ClientRemoteTabs::from_record(id, record)
            };
            remote_tabs.push(crt);
        }

        // We want to keep the mutex for as short as possible
        let local_tabs = {
            let mut storage = self.store.storage.lock().unwrap();
            storage.replace_remote_tabs(remote_tabs)?;
            storage.prepare_local_tabs_for_upload()
        };
        let outgoing = if let Some(local_tabs) = local_tabs {
            let (client_name, device_type) = self
                .remote_clients
                .get(&local_id)
                .map(|client| {
                    (
                        client.device_name.clone(),
                        client.device_type.unwrap_or(DeviceType::Unknown),
                    )
                })
                .unwrap_or_else(|| (String::new(), DeviceType::Unknown));
            let local_record = ClientRemoteTabs {
                client_id: local_id,
                client_name,
                device_type,
                remote_tabs: local_tabs.to_vec(),
            };
            log::trace!("outgoing {:?}", local_record);
            Some(local_record.to_record())
        } else {
            None
        };
        Ok(outgoing)
    }

    pub fn sync_finished(
        &mut self,
        new_timestamp: ServerTimestamp,
        records_synced: &[Guid],
    ) -> Result<()> {
        log::info!(
            "sync completed after uploading {} records",
            records_synced.len()
        );
        self.last_sync = Some(new_timestamp);
        Ok(())
    }

    pub fn reset(&mut self, assoc: EngineSyncAssociation) -> Result<()> {
        self.remote_clients.clear();
        self.sync_store_assoc = assoc;
        self.last_sync = None;
        self.store.storage.lock().unwrap().wipe_remote_tabs()?;
        Ok(())
    }

    pub fn wipe(&mut self) -> Result<()> {
        self.reset(EngineSyncAssociation::Disconnected)?;
        // not clear why we need to wipe the local tabs - the app is just going
        // to re-add them?
        self.store.storage.lock().unwrap().wipe_local_tabs();
        Ok(())
    }

    pub fn get_sync_assoc(&self) -> &EngineSyncAssociation {
        &self.sync_store_assoc
    }
}
