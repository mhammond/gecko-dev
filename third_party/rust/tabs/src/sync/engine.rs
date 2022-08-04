/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::sync::record::TabsRecord;
use crate::sync::sync_impl::TabsSyncImpl;
use crate::TabsStore;
use anyhow::Result;
use interrupt_support::NeverInterrupts;
use std::sync::{Arc, Mutex, Weak};
use sync15::{
    clients::{self},
    sync_multiple, telemetry, CollectionRequest, EngineSyncAssociation, IncomingChangeset,
    MemoryCachedState, OutgoingChangeset, Payload, ServerTimestamp, SyncEngine, SyncEngineId,
};
use sync_guid::Guid;

// Our "sync manager" will use whatever is stashed here.
lazy_static::lazy_static! {
    // Mutex: just taken long enough to update the inner stuff
    static ref STORE_FOR_MANAGER: Mutex<Weak<TabsStore>> = Mutex::new(Weak::new());
}

/// Called by the sync manager to get a sync engine via the store previously
/// registered with the sync manager.
pub fn get_registered_sync_engine(engine_id: &SyncEngineId) -> Option<Box<dyn SyncEngine>> {
    let weak = STORE_FOR_MANAGER.lock().unwrap();
    match weak.upgrade() {
        None => None,
        Some(store) => match engine_id {
            SyncEngineId::Tabs => Some(Box::new(TabsEngine::new(Arc::clone(&store)))),
            // panicing here seems reasonable - it's a static error if this
            // it hit, not something that runtime conditions can influence.
            _ => unreachable!("can't provide unknown engine: {}", engine_id),
        },
    }
}

impl TabsStore {
    pub fn reset(self: Arc<Self>) -> Result<()> {
        let engine = TabsEngine::new(Arc::clone(&self));
        engine.reset(&EngineSyncAssociation::Disconnected)?;
        Ok(())
    }

    /// A convenience wrapper around sync_multiple.
    pub fn sync(
        self: Arc<Self>,
        key_id: String,
        access_token: String,
        sync_key: String,
        tokenserver_url: String,
        local_id: String,
    ) -> Result<String> {
        let mut mem_cached_state = MemoryCachedState::default();
        let engine = TabsEngine::new(Arc::clone(&self));

        // Since we are syncing without the sync manager, there's no
        // command processor, therefore no clients engine, and in
        // consequence `TabsStore::prepare_for_sync` is never called
        // which means our `local_id` will never be set.
        // Do it here.
        engine.sync_impl.lock().unwrap().local_id = local_id;

        let storage_init = &sync15::Sync15StorageClientInit {
            key_id,
            access_token,
            tokenserver_url: url::Url::parse(tokenserver_url.as_str())?,
        };
        let root_sync_key = &sync15::KeyBundle::from_ksync_base64(sync_key.as_str())?;

        let mut result = sync_multiple(
            &[&engine],
            &mut None,
            &mut mem_cached_state,
            storage_init,
            root_sync_key,
            &NeverInterrupts,
            None,
        );

        // for b/w compat reasons, we do some dances with the result.
        // XXX - note that this means telemetry isn't going to be reported back
        // to the app - we need to check with lockwise about whether they really
        // need these failures to be reported or whether we can loosen this.
        if let Err(e) = result.result {
            return Err(e.into());
        }
        match result.engine_results.remove("tabs") {
            None | Some(Ok(())) => Ok(serde_json::to_string(&result.telemetry)?),
            Some(Err(e)) => Err(e.into()),
        }
    }

    // This allows the embedding app to say "make this instance available to
    // the sync manager". The implementation is more like "offer to sync mgr"
    // (thereby avoiding us needing to link with the sync manager) but
    // `register_with_sync_manager()` is logically what's happening so that's
    // the name it gets.
    pub fn register_with_sync_manager(self: Arc<Self>) {
        let mut state = STORE_FOR_MANAGER.lock().unwrap();
        *state = Arc::downgrade(&self);
    }
}

pub struct TabsEngine {
    pub sync_impl: Mutex<TabsSyncImpl>,
}

impl TabsEngine {
    pub fn new(store: Arc<TabsStore>) -> Self {
        Self {
            sync_impl: Mutex::new(TabsSyncImpl::new(store)),
        }
    }
}

impl SyncEngine for TabsEngine {
    fn collection_name(&self) -> std::borrow::Cow<'static, str> {
        "tabs".into()
    }

    fn prepare_for_sync(&self, get_client_data: &dyn Fn() -> clients::ClientData) -> Result<()> {
        Ok(self
            .sync_impl
            .lock()
            .unwrap()
            .prepare_for_sync(get_client_data())?)
    }

    fn apply_incoming(
        &self,
        inbound: Vec<IncomingChangeset>,
        telem: &mut telemetry::Engine,
    ) -> Result<OutgoingChangeset> {
        assert_eq!(inbound.len(), 1, "only requested one set of records");
        let inbound = inbound.into_iter().next().unwrap();
        let mut incoming_telemetry = telemetry::EngineIncoming::new();
        let mut incoming_records = Vec::with_capacity(inbound.changes.len());

        for incoming in inbound.changes {
            let record = match TabsRecord::from_payload(incoming.0) {
                Ok(record) => record,
                Err(e) => {
                    log::warn!("Error deserializing incoming record: {}", e);
                    incoming_telemetry.failed(1);
                    continue;
                }
            };
            incoming_records.push(record);
        }

        let outgoing_record = self
            .sync_impl
            .lock()
            .unwrap()
            .apply_incoming(incoming_records)?;

        let mut outgoing = OutgoingChangeset::new("tabs", inbound.timestamp);
        if let Some(outgoing_record) = outgoing_record {
            let payload = Payload::from_record(outgoing_record)?;
            outgoing.changes.push(payload);
        }
        telem.incoming(incoming_telemetry);
        Ok(outgoing)
    }

    fn sync_finished(
        &self,
        new_timestamp: ServerTimestamp,
        records_synced: Vec<Guid>,
    ) -> Result<()> {
        Ok(self
            .sync_impl
            .lock()
            .unwrap()
            .sync_finished(new_timestamp, &records_synced)?)
    }

    fn get_collection_requests(
        &self,
        server_timestamp: ServerTimestamp,
    ) -> Result<Vec<CollectionRequest>> {
        let since = self.sync_impl.lock().unwrap().last_sync.unwrap_or_default();
        Ok(if since == server_timestamp {
            vec![]
        } else {
            vec![CollectionRequest::new("tabs").full().newer_than(since)]
        })
    }

    fn get_sync_assoc(&self) -> Result<EngineSyncAssociation> {
        Ok(self.sync_impl.lock().unwrap().get_sync_assoc().clone())
    }

    fn reset(&self, assoc: &EngineSyncAssociation) -> Result<()> {
        Ok(self.sync_impl.lock().unwrap().reset(assoc.clone())?)
    }

    fn wipe(&self) -> Result<()> {
        Ok(self.sync_impl.lock().unwrap().wipe()?)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use serde_json::json;
    use sync15_traits::DeviceType;

    #[test]
    fn test_incoming_tabs() {
        env_logger::try_init().ok();

        let engine = TabsEngine::new(Arc::new(TabsStore::new_with_mem_path("test")));

        let records = vec![
            json!({
                "id": "device-no-tabs",
                "clientName": "device with no tabs",
                "tabs": [],
            }),
            json!({
                "id": "device-with-a-tab",
                "clientName": "device with a tab",
                "tabs": [{
                    "title": "the title",
                    "urlHistory": [
                        "https://mozilla.org/"
                    ],
                    "icon": "https://mozilla.org/icon",
                    "lastUsed": 1643764207
                }]
            }),
            // This has the main payload as OK but the tabs part invalid.
            json!({
                "id": "device-with-invalid-tab",
                "clientName": "device with a tab",
                "tabs": [{
                    "foo": "bar",
                }]
            }),
            // We want this to be a valid payload but an invalid tab - so it needs an ID.
            json!({
                "id": "invalid-tab",
                "foo": "bar"
            }),
        ];

        let mut incoming = IncomingChangeset::new(engine.collection_name(), ServerTimestamp(0));
        for record in records {
            let payload = Payload::from_json(record).unwrap();
            incoming.changes.push((payload, ServerTimestamp(0)));
        }
        let outgoing = engine
            .apply_incoming(vec![incoming], &mut telemetry::Engine::new("tabs"))
            .expect("Should apply incoming and stage outgoing records");

        assert!(outgoing.changes.is_empty());

        // now check the store has what we think it has.
        let sync_impl = engine.sync_impl.lock().unwrap();
        let mut storage = sync_impl.store.storage.lock().unwrap();
        let mut crts = storage.get_remote_tabs().expect("should work");
        crts.sort_by(|a, b| a.client_name.partial_cmp(&b.client_name).unwrap());
        assert_eq!(crts.len(), 2, "we currently include devices with no tabs");
        let crt = &crts[0];
        assert_eq!(crt.client_name, "device with a tab");
        assert_eq!(crt.device_type, DeviceType::Unknown);
        assert_eq!(crt.remote_tabs.len(), 1);
        assert_eq!(crt.remote_tabs[0].title, "the title");

        let crt = &crts[1];
        assert_eq!(crt.client_name, "device with no tabs");
        assert_eq!(crt.device_type, DeviceType::Unknown);
        assert_eq!(crt.remote_tabs.len(), 0);
    }

    #[test]
    fn test_sync_manager_registration() {
        let store = Arc::new(TabsStore::new_with_mem_path("test"));
        assert_eq!(Arc::strong_count(&store), 1);
        assert_eq!(Arc::weak_count(&store), 0);
        Arc::clone(&store).register_with_sync_manager();
        assert_eq!(Arc::strong_count(&store), 1);
        assert_eq!(Arc::weak_count(&store), 1);
        let registered = STORE_FOR_MANAGER
            .lock()
            .unwrap()
            .upgrade()
            .expect("should upgrade");
        assert!(Arc::ptr_eq(&store, &registered));
        drop(registered);
        // should be no new references
        assert_eq!(Arc::strong_count(&store), 1);
        assert_eq!(Arc::weak_count(&store), 1);
        // dropping the registered object should drop the registration.
        drop(store);
        assert!(STORE_FOR_MANAGER.lock().unwrap().upgrade().is_none());
    }
}
