/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(feature = "full-sync")]
pub(crate) mod engine;

mod record;

#[cfg(any(feature = "full-sync", feature = "bridged-sync"))]
mod sync_impl;

#[cfg(feature = "bridged-sync")]
pub(crate) mod bridge;

// Our UDL gives the store certain sync-specific functions, but their availability
// depends on which sync features are enabled - so we must provide stubs to handle
// then the features don't allow the "real" implementation.
// (Ideally we could teach uniffi about features)
#[cfg(not(feature = "bridged-sync"))]
pub(crate) mod bridge {
    use sync_guid::Guid as SyncGuid;

    pub struct TabsBridgedEngine {}

    impl TabsBridgedEngine {
        pub fn last_sync(&self) -> crate::error::Result<i64> {
            unreachable!();
        }

        pub fn set_last_sync(&self, _last_sync: i64) -> crate::error::Result<()> {
            unreachable!();
        }

        pub fn sync_id(&self) -> crate::error::Result<Option<String>> {
            unreachable!();
        }

        pub fn reset_sync_id(&self) -> crate::error::Result<String> {
            unreachable!();
        }

        pub fn ensure_current_sync_id(&self, _sync_id: &str) -> crate::error::Result<String> {
            unreachable!();
        }

        pub fn prepare_for_sync(&self, _client_data: &str) -> crate::error::Result<()> {
            unreachable!();
        }

        pub fn sync_started(&self) -> crate::error::Result<()> {
            unreachable!();
        }

        pub fn store_incoming(&self, _incoming: Vec<String>) -> crate::error::Result<()> {
            unreachable!();
        }

        pub fn apply(&self) -> crate::error::Result<Vec<String>> {
            unreachable!();
        }

        pub fn set_uploaded(
            &self,
            _server_modified_millis: i64,
            _guids: Vec<SyncGuid>,
        ) -> crate::error::Result<()> {
            unreachable!();
        }

        pub fn sync_finished(&self) -> crate::error::Result<()> {
            unreachable!();
        }

        pub fn reset(&self) -> crate::error::Result<()> {
            unreachable!();
        }

        pub fn wipe(&self) -> crate::error::Result<()> {
            unreachable!();
        }
    }

    impl crate::TabsStore {
        // Returns a bridged sync engine for Desktop for this store.
        pub fn bridged_engine(self: std::sync::Arc<Self>) -> std::sync::Arc<TabsBridgedEngine> {
            log::error!("bridged_engine: feature not enabled");
            // XXX - there's no good error to return here.
            unreachable!();
        }
    }
}

// Similarly, when full-sync isn't enabled we need stub versions for these functions.
#[cfg(not(feature = "full-sync"))]
impl crate::TabsStore {
    pub fn reset(self: std::sync::Arc<Self>) -> crate::error::Result<()> {
        log::error!("reset: feature not enabled");
        Err(crate::error::TabsError::SyncAdapterError(
            "reset".to_string(),
        ))
    }

    pub fn sync(
        self: std::sync::Arc<Self>,
        _key_id: String,
        _access_token: String,
        _sync_key: String,
        _tokenserver_url: String,
        _local_id: String,
    ) -> crate::error::Result<String> {
        log::error!("sync: feature not enabled");
        Err(crate::error::TabsError::SyncAdapterError(
            "sync".to_string(),
        ))
    }

    pub fn register_with_sync_manager(self: std::sync::Arc<Self>) {
        log::error!("register_with_sync_manager: feature not enabled");
    }
}
