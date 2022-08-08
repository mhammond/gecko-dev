/* Any copyright is dedicated to the Public Domain.
   http://creativecommons.org/publicdomain/zero/1.0/ */

const { TabEngine } = ChromeUtils.import(
  "resource://services-sync/engines/tabs.js"
);
const { WBORecord } = ChromeUtils.import("resource://services-sync/record.js");
const { Service } = ChromeUtils.import("resource://services-sync/service.js");

async function getMocks() {
  let engine = new TabEngine(Service);
  await engine.initialize();
  let bridge = engine._bridge;
  engine._provider.shouldSkipWindow = mockShouldSkipWindow;
  return [engine, bridge];
}

add_task(async function test_tab_engine_skips_incoming_local_record() {
  _("Ensure incoming records that match local client ID are never applied.");
  let [engine, bridge] = await getMocks();
  let localID = engine.service.clientsEngine.localID;
  let apply = bridge.storeIncoming;
  let applied = [];

  bridge.storeIncoming = async function(incomingAsJson) {
    let record = JSON.parse(incomingAsJson[0]);
    Assert.ok(record.id);
    applied.push(record);
    apply.call(bridge, incomingAsJson);
  };

  let collection = new ServerCollection();

  _("Creating remote tab record with local client ID");
  let localRecord = encryptPayload({
    id: localID,
    clientName: "local",
    tabs: [],
  });
  collection.insert(localID, localRecord);

  _("Creating remote tab record with a different client ID");
  let remoteID = "different";
  let remoteRecord = encryptPayload({
    id: remoteID,
    clientName: "not local",
    tabs: [],
  });
  collection.insert(remoteID, remoteRecord);

  _("Setting up Sync server");
  let server = sync_httpd_setup({
    "/1.1/foo/storage/tabs": collection.handler(),
  });

  await SyncTestingInfrastructure(server);

  let syncID = await engine.resetLocalSyncID();
  let meta_global = Service.recordManager.set(
    engine.metaURL,
    new WBORecord(engine.metaURL)
  );
  meta_global.payload.engines = { tabs: { version: engine.version, syncID } };

  await generateNewKeys(Service.collectionKeys);

  let promiseFinished = new Promise(resolve => {
    let syncFinish = engine._syncFinish;
    engine._syncFinish = async function() {
      let remoteTabs = await engine._rustStore.getAll();
      console.log(remoteTabs);
      equal(
        remoteTabs.length,
        1,
        "Remote client record was applied and local wasn't"
      );
      let record = remoteTabs[0];
      equal(record.clientId, remoteID, "Remote client ID matches");

      let clients = await engine.getAllClients();
      // XXX - test the shape of getAllClients
      // client.id, client.type, client.tabs

      await syncFinish.call(engine);
      resolve();
    };
  });

  _("Start sync");
  await engine._sync();
  await promiseFinished;
});

add_task(async function test_create() {
  let [engine, bridge] = getMocks();
  Assert.ok(bridge);

  _("Create a first record");
  let rec = encryptPayload({
    id: "id1",
    clientName: "clientName1",
    tabs: [
      {
        title: "title",
        urlHistory: ["http://example.com/"],
        icon: "",
        lastUsed: 2,
      },
      {
        title: "title1",
        urlHistory: ["http://example.com/"],
        icon: "",
        lastUsed: 2,
      },
    ],
  });
  await bridge.storeIncoming([JSON.stringify(rec)]);
  deepEqual(bridge._remoteClients.id1, { lastModified: 1000, foo: "bar" });

  _("Create a second record");
  rec = encryptPayload({
    id: "id2",
    clientName: "clientName2",
    tabs: [
      {
        title: "title2",
        urlHistory: ["http://someotherexample.com/"],
        icon: "",
        lastUsed: 3,
      },
    ],
  });
  await bridge.storeIncoming([JSON.stringify(rec)]);
  deepEqual(bridge._remoteClients.id2, { lastModified: 2000, foo2: "bar2" });

  _("Create a third record");
  rec = encryptPayload({
    id: "id3",
    clientName: "clientName3",
    tabs: [
      {
        title: "title3",
        urlHistory: ["http://example.com/"],
        icon: "",
        lastUsed: 2,
      },
    ],
  });
  await bridge.storeIncoming([JSON.stringify(rec)]);
  deepEqual(bridge._remoteClients.id3, { lastModified: 3000, foo3: "bar3" });
});
