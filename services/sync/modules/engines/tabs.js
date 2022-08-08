/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

var EXPORTED_SYMBOLS = ["TabEngine", "TabSetRecord"];

const TABS_TTL = 31622400; // 366 days (1 leap year).
const TAB_ENTRIES_LIMIT = 5; // How many URLs to include in tab history.

const { XPCOMUtils } = ChromeUtils.importESModule(
  "resource://gre/modules/XPCOMUtils.sys.mjs"
);
const { Log } = ChromeUtils.import("resource://gre/modules/Log.jsm");
const { Store, SyncEngine, Tracker } = ChromeUtils.import(
  "resource://services-sync/engines.js"
);
const { CryptoWrapper } = ChromeUtils.import(
  "resource://services-sync/record.js"
);
const { Svc, Utils } = ChromeUtils.import("resource://services-sync/util.js");
const {
  LOGIN_SUCCEEDED,
  SCORE_INCREMENT_SMALL,
  STATUS_OK,
  URI_LENGTH_MAX,
} = ChromeUtils.import("resource://services-sync/constants.js");
const { CommonUtils } = ChromeUtils.import(
  "resource://services-common/utils.js"
);
const { Async } = ChromeUtils.import("resource://services-common/async.js");

const { SyncRecord, SyncTelemetry } = ChromeUtils.import(
  "resource://services-sync/telemetry.js"
);

const { BridgedEngine, LogAdapter } = ChromeUtils.import(
  "resource://services-sync/bridged_engine.js"
);

const FAR_FUTURE = 4102405200000; // 2100/01/01

const lazy = {};

ChromeUtils.defineModuleGetter(
  lazy,
  "PrivateBrowsingUtils",
  "resource://gre/modules/PrivateBrowsingUtils.jsm"
);

ChromeUtils.defineESModuleGetters(lazy, {
  PlacesUtils: "resource://gre/modules/PlacesUtils.sys.mjs",
});

XPCOMUtils.defineLazyModuleGetters(lazy, {
  ClientRemoteTabs: "resource://gre/modules/Tabs.jsm",
  NimbusFeatures: "resource://nimbus/ExperimentAPI.jsm",
  RemoteTabRecord: "resource://gre/modules/Tabs.jsm",
  TabsStore: "resource://gre/modules/Tabs.jsm",
});

XPCOMUtils.defineLazyPreferenceGetter(
  lazy,
  "TABS_FILTERED_SCHEMES",
  "services.sync.engine.tabs.filteredSchemes",
  "",
  null,
  val => {
    return new Set(val.split("|"));
  }
);

XPCOMUtils.defineLazyPreferenceGetter(
  lazy,
  "SYNC_AFTER_DELAY_MS",
  "services.sync.syncedTabs.syncDelayAfterTabChange",
  0
);

XPCOMUtils.defineLazyGetter(lazy, "rustStore", async () => {
  let path = PathUtils.join(PathUtils.profileDir, "synced-tabs.db");
  return await lazy.TabsStore.init(path);
});

// A "bridged engine" to our tabs component.
function TabEngine(service) {
  BridgedEngine.call(this, "Tabs", service);

  // XXX - this logger stuff probably doesn't work?
  let app_services_logger = Cc["@mozilla.org/appservices/logger;1"].getService(
    Ci.mozIAppServicesLogger
  );
  let logger_target = "app-services:tabs:sync";
  app_services_logger.register(logger_target, new LogAdapter(this._log));
}

TabEngine.prototype = {
  __proto__: BridgedEngine.prototype,
  _providerObj: TabProvider,
  _trackerObj: TabTracker,
  syncPriority: 3,

  get _provider() {
    let provider = new this._providerObj(this.Name, this);
    this.__defineGetter__("_provider", () => provider);
    return provider;
  },

  async _syncStartup() {
    // Tell the bridged engine about clients.
    // This is the same shape as ClientData in app-services.
    let clientData = {
      local_client_id: this.service.clientsEngine.localID,
      recent_clients: {},
    };
    let clientsEngine = this.service.clientsEngine;
    for (let remoteClient of clientsEngine.remoteClients) {
      let id = remoteClient.id;
      if (!id) {
        throw "no id?";
      }
      let client = {
        fxa_device_id: remoteClient.fxaDeviceId,
        // device_name and device_type are soft-deprecated - every client
        // prefers what's in the FxA record. But fill them correctly anyway.
        device_name: clientsEngine.getClientName(id),
        device_type: clientsEngine.getClientType(id),
      };
      clientData.recent_clients[id] = client;
    }
    await this._bridge.prepareForSync(JSON.stringify(clientData));

    let tabs = await this._provider.getAllTabs(true);
    await this._rustStore.setLocalTabs(tabs);
    await this._bridge.setLastSync(0);
    await super._syncStartup();
  },

  async initialize() {
    await SyncEngine.prototype.initialize.call(this);

    let rustStore = await lazy.rustStore;
    this._bridge = await rustStore.bridgedEngine();
    // XXX - This is particularly useful for accessing the store in tests
    this._rustStore = rustStore;
    this._log.info("Got a bridged engine!");

    // Reset the client on every startup so that we fetch recent tabs.
    await this._resetClient();
  },

  // XXX - this should be the only reference to the tracker. We can probably
  // just use a bool on the engine and drop the tracker entirely?
  async getChangedIDs() {
    // No need for a proper timestamp (no conflict resolution needed).
    let changedIDs = {};
    if (this._tracker.modified) {
      changedIDs[this.service.clientsEngine.localID] = 0;
    }
    return changedIDs;
  },

  // API for use by Sync UI code to give user choices of tabs to open.
  async getAllClients() {
    let remoteTabs = await this._rustStore.getAll();
    let remoteClientTabs = [];
    for (let remoteClient of this.service.clientsEngine.remoteClients) {
      // we want the tabs from the rust engine but the shape from clients engine
      let rustClient = remoteTabs.find(
        x => x.clientId === remoteClient.fxaDeviceId
      );
      if (!rustClient) {
        continue;
      }

      let client = {
        tabs: rustClient.remoteTabs,
        ...remoteClient,
      };
      remoteClientTabs.push(client);
    }
    return remoteClientTabs;
  },

  // This seems to only be used by services/sync/tps/extensions/.../tabs.jsm
  // Potential to remove and/or move to that test script?
  // (Yeah - remove and copy it to tps!)
  getClientById(id) {
    return this._store._remoteClients[id];
  },

  async removeClientData() {
    // This needs thought?
    let url = this.engineURL + "/" + this.service.clientsEngine.localID;
    await this.service.resource(url).delete();
  },

  async trackRemainingChanges() {
    if (this._modified.count() > 0) {
      this._tracker.modified = true;
    }
  },

  // Support for "quick writes"
  // XXX - needs love!
  _engineLock: Utils.lock,
  _engineLocked: false,

  // Tabs has a special lock to help support its "quick write"
  get locked() {
    return this._engineLocked;
  },
  lock() {
    if (this._engineLocked) {
      return false;
    }
    this._engineLocked = true;
    return true;
  },
  unlock() {
    this._engineLocked = false;
  },

  // Quickly do a POST of our current tabs if possible.
  // This does things that would be dangerous for other engines - eg, posting
  // without checking what's on the server could cause data-loss for other
  // engines, but because each device exclusively owns exactly 1 tabs record
  // with a known ID, it's safe here.
  async quickWrite() {
    if (!this.enabled) {
      // this should be very rare, and only if tabs are disabled after the
      // timer is created.
      this._log.info("Can't do a quick-sync as tabs is disabled");
      return;
    }
    // This quick-sync doesn't drive the login state correctly, so just
    // decline to sync if out status is bad
    if (this.service.status.checkSetup() != STATUS_OK) {
      this._log.info(
        "Can't do a quick-sync due to the service status",
        this.service.status.toString()
      );
      return;
    }
    if (!this.service.serverConfiguration) {
      this._log.info("Can't do a quick sync before the first full sync");
      return;
    }
    try {
      await this._engineLock("tabs.js: quickWrite", async () => {
        // We want to restore the lastSync timestamp when complete so next sync
        // takes tabs written by other devices since our last real sync.
        // And for this POST we don't want the protections offered by
        // X-If-Unmodified-Since - we want the POST to work even if the remote
        // has moved on and we will catch back up next full sync.
        const origLastSync = await this.getLastSync();
        // XXX -  Quick sync is broken not detecting changes to submit
        await this._bridge.setLastSync(FAR_FUTURE);
        try {
          await this._doQuickWrite();
        } finally {
          await this._bridge.setLastSync(Math.floor(origLastSync / 1000));
        }
      })();
    } catch (ex) {
      if (!Utils.isLockException(ex)) {
        throw ex;
      }
      this._log.info(
        "Can't do a quick-write as another tab sync is in progress"
      );
    }
  },

  // The guts of the quick-write sync, after we've taken the lock, checked
  // the service status etc.
  async _doQuickWrite() {
    // We need to track telemetry for these syncs too!
    const name = "tabs";
    let telemetryRecord = new SyncRecord(
      SyncTelemetry.allowedEngines,
      "quick-write"
    );
    telemetryRecord.onEngineStart(name);
    try {
      Async.checkAppReady();
      // tracking the modified items is normally done by _syncStartup(),
      // but we don't call that so we don't do the meta/global dances -
      // these dances would be very important for any other engine, but
      // we can avoid it for tabs because of the lack of reconcilliation.
      let tabs = await this._provider.getAllTabs(true);
      await this._rustStore.setLocalTabs(tabs);
      this._modified.replace(await this.pullChanges());
      this._tracker.clearChangedIDs();
      this._tracker.resetScore();

      // now just the "upload" part of a sync.
      Async.checkAppReady();
      await this._uploadOutgoing();
      telemetryRecord.onEngineApplied(name, 1);
      telemetryRecord.onEngineStop(name, null);
    } catch (ex) {
      telemetryRecord.onEngineStop(name, ex);
    } finally {
      // The top-level sync is never considered to fail here, just the engine
      telemetryRecord.finished(null);
      SyncTelemetry.takeTelemetryRecord(telemetryRecord);
    }
  },

  async _sync() {
    try {
      await this._engineLock("tabs.js: fullSync", async () => {
        await super._sync();
      })();
    } catch (ex) {
      if (!Utils.isLockException(ex)) {
        throw ex;
      }
      this._log.info(
        "Can't do full tabs sync as a quick-write is currently running"
      );
    }
  },
};

function TabProvider() {}

TabProvider.prototype = {
  getWindowEnumerator() {
    return Services.wm.getEnumerator("navigator:browser");
  },

  shouldSkipWindow(win) {
    return win.closed || lazy.PrivateBrowsingUtils.isWindowPrivate(win);
  },

  async getAllTabs(filter) {
    let allTabs = [];
    let iconPromises = [];

    for (let win of this.getWindowEnumerator()) {
      if (this.shouldSkipWindow(win)) {
        continue;
      }

      for (let tab of win.gBrowser.tabs) {
        // Note that we used to sync "tab history" (ie, the "back button") state,
        // but in practice this hasn't been used - only the current URI is of
        // interest to clients.
        // We stopped recording this in bug 1783991.
        if (!tab?.linkedBrowser) {
          continue;
        }
        let acceptable = !filter
          ? url => url
          : url =>
              url &&
              !lazy.TABS_FILTERED_SCHEMES.has(Services.io.extractScheme(url));

        let url = tab.linkedBrowser.currentURI?.spec;
        // We ignore the tab completely if the current entry url is
        // not acceptable (we need something accurate to open).
        if (!acceptable(url)) {
          continue;
        }

        if (url.length > URI_LENGTH_MAX) {
          this._log.trace("Skipping over-long URL.");
          continue;
        }

        let thisTab = {
          title: tab.linkedBrowser.contentTitle || "",
          urlHistory: [url],
          icon: "",
          // XXX - need to change this to be in ms
          //lastUsed: Math.floor((tab.lastAccessed || 0) / 1000),
          lastUsed: Math.floor(tab.lastAccessed || 0),
        };
        allTabs.push(thisTab);
        // Use the favicon service for the icon url - we can wait for the promises at the end.
        let iconPromise = lazy.PlacesUtils.promiseFaviconData(url)
          .then(iconData => {
            thisTab.icon = iconData.uri.spec;
          })
          .catch(ex => {
            this._log.trace(
              `Failed to fetch favicon for ${url}`,
              thisTab.urlHistory[0]
            );
          });
        iconPromises.push(iconPromise);
      }
    }

    await Promise.allSettled(iconPromises);

    return allTabs.sort(function(a, b) {
      return b.lastUsed - a.lastUsed;
    });
  },
};

function TabTracker(name, engine) {
  Tracker.call(this, name, engine);

  // Make sure "this" pointer is always set correctly for event listeners.
  this.onTab = Utils.bind2(this, this.onTab);
  this._unregisterListeners = Utils.bind2(this, this._unregisterListeners);
}
TabTracker.prototype = {
  __proto__: Tracker.prototype,

  QueryInterface: ChromeUtils.generateQI(["nsIObserver"]),

  clearChangedIDs() {
    this.modified = false;
  },

  // We do not track TabSelect because that almost always triggers
  // the web progress listeners (onLocationChange), which we already track
  _topics: ["TabOpen", "TabClose"],

  _registerListenersForWindow(window) {
    this._log.trace("Registering tab listeners in window");
    for (let topic of this._topics) {
      window.addEventListener(topic, this.onTab);
    }
    window.addEventListener("unload", this._unregisterListeners);
    // If it's got a tab browser we can listen for things like navigation.
    if (window.gBrowser) {
      window.gBrowser.addProgressListener(this);
    }
  },

  _unregisterListeners(event) {
    this._unregisterListenersForWindow(event.target);
  },

  _unregisterListenersForWindow(window) {
    this._log.trace("Removing tab listeners in window");
    window.removeEventListener("unload", this._unregisterListeners);
    for (let topic of this._topics) {
      window.removeEventListener(topic, this.onTab);
    }
    if (window.gBrowser) {
      window.gBrowser.removeProgressListener(this);
    }
  },

  onStart() {
    Svc.Obs.add("domwindowopened", this.asyncObserver);
    for (let win of Services.wm.getEnumerator("navigator:browser")) {
      this._registerListenersForWindow(win);
    }
  },

  onStop() {
    Svc.Obs.remove("domwindowopened", this.asyncObserver);
    for (let win of Services.wm.getEnumerator("navigator:browser")) {
      this._unregisterListenersForWindow(win);
    }
  },

  async observe(subject, topic, data) {
    switch (topic) {
      case "domwindowopened":
        let onLoad = () => {
          subject.removeEventListener("load", onLoad);
          // Only register after the window is done loading to avoid unloads.
          this._registerListenersForWindow(subject);
        };

        // Add tab listeners now that a window has opened.
        subject.addEventListener("load", onLoad);
        break;
    }
  },

  onTab(event) {
    if (event.originalTarget.linkedBrowser) {
      let browser = event.originalTarget.linkedBrowser;
      if (
        lazy.PrivateBrowsingUtils.isBrowserPrivate(browser) &&
        !lazy.PrivateBrowsingUtils.permanentPrivateBrowsing
      ) {
        this._log.trace("Ignoring tab event from private browsing.");
        return;
      }
    }
    this._log.trace("onTab event: " + event.type);

    switch (event.type) {
      case "TabOpen":
        /* We do not have a reliable way of checking the URI on the TabOpen
         * so we will rely on the other methods (onLocationChange, getAllTabs)
         * to filter these when going through sync
         */
        this.callScheduleSync(SCORE_INCREMENT_SMALL);
        break;
      case "TabClose":
        // If event target has `linkedBrowser`, the event target can be assumed <tab> element.
        // Else, event target is assumed <browser> element, use the target as it is.
        const tab = event.target.linkedBrowser || event.target;

        // TabClose means the tab has already loaded and we can check the URI
        // and ignore if it's a scheme we don't care about
        if (lazy.TABS_FILTERED_SCHEMES.has(tab.currentURI.scheme)) {
          return;
        }
        this.callScheduleSync(SCORE_INCREMENT_SMALL);
        break;
    }
  },

  // web progress listeners.
  onLocationChange(webProgress, request, locationURI, flags) {
    // We only care about top-level location changes which are not in the same
    // document.
    if (
      flags & Ci.nsIWebProgressListener.LOCATION_CHANGE_SAME_DOCUMENT ||
      flags & Ci.nsIWebProgressListener.LOCATION_CHANGE_RELOAD ||
      !webProgress.isTopLevel ||
      !locationURI
    ) {
      return;
    }

    // We can't filter out tabs that we don't sync here, because we might be
    // navigating from a tab that we *did* sync to one we do not, and that
    // tab we *did* sync should no longer be synced.
    this.callScheduleSync();
  },

  callScheduleSync(scoreIncrement) {
    this.modified = true;
    let { scheduler } = this.engine.service;
    let delayInMs = lazy.SYNC_AFTER_DELAY_MS;

    // We have this check to determine if the experiment is enabled and wants
    // to override the default values (whether to lengthen the delay or disable completely)
    const override = lazy.NimbusFeatures.syncAfterTabChange.getVariable(
      "syncDelayAfterTabChangeOverride"
    );
    if (override) {
      delayInMs = lazy.NimbusFeatures.syncAfterTabChange.getVariable(
        "syncDelayAfterTabChange"
      );
    }

    // If we are part of the experiment don't use score here
    // and instead schedule a sync once we detect a tab change
    // to ensure the server always has the most up to date tabs]
    if (
      delayInMs > 0 &&
      scheduler.numClients > 1 // Only schedule quick syncs for single client users
    ) {
      if (this.tabsQuickWriteTimer) {
        this._log.debug(
          "Detected a tab change, but a quick-write is already scheduled"
        );
        return;
      }
      this._log.debug(
        "Detected a tab change: scheduling a quick-write in " + delayInMs + "ms"
      );
      CommonUtils.namedTimer(
        () => {
          this._log.trace("tab quick-sync timer fired.");
          this.engine
            .quickWrite()
            .then(() => {
              this._log.trace("tab quick-sync done.");
            })
            .catch(ex => {
              this._log.error("tab quick-sync failed.", ex);
            });
        },
        delayInMs,
        this,
        "tabsQuickWriteTimer"
      );
    } else if (scoreIncrement) {
      this.score += scoreIncrement;
    }
  },
};
