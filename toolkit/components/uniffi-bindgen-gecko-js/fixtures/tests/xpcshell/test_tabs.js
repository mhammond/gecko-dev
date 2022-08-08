/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const { TabsStore, RemoteTabRecord, ClientRemoteTabs } = ChromeUtils.import(
  "resource://gre/modules/components-utils/Tabs.jsm"
);

add_task(async function() {
  const tabStore = await TabsStore.init("temp-tabs.db");
  Assert.ok(tabStore);

  let remoteTabs = [];
  let tab1 = new RemoteTabRecord("tab1", [], "", Date.now());
  remoteTabs.push(tab1);
  await tabStore.setLocalTabs(remoteTabs);
});
