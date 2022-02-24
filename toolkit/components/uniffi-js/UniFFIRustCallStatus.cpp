/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim:set ts=2 sw=2 sts=2 et cindent: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "mozilla/dom/UniFFIRustCallStatus.h"
#include "mozilla/dom/UniFFIRustCallStatusBinding.h"

namespace mozilla::dom {

// Only needed for refcounted objects.
NS_IMPL_CYCLE_COLLECTION_WRAPPERCACHE_0(UniFFIRustCallStatus)
NS_IMPL_CYCLE_COLLECTING_ADDREF(UniFFIRustCallStatus)
NS_IMPL_CYCLE_COLLECTING_RELEASE(UniFFIRustCallStatus)
NS_INTERFACE_MAP_BEGIN_CYCLE_COLLECTION(UniFFIRustCallStatus)
  NS_WRAPPERCACHE_INTERFACE_MAP_ENTRY
  NS_INTERFACE_MAP_ENTRY(nsISupports)
NS_INTERFACE_MAP_END

UniFFIRustCallStatus::UniFFIRustCallStatus(nsCOMPtr<nsIGlobalObject> aGlobal)
    : mGlobal(aGlobal), mCode(0), mBuf() {}

already_AddRefed<UniFFIRustCallStatus> UniFFIRustCallStatus::Constructor(
    const GlobalObject& aGlobal) {
  nsCOMPtr<nsIGlobalObject> global = do_QueryInterface(aGlobal.GetAsSupports());
  if (!global) {
    // do_QueryInterface somehow failed even though we called in on a
    // GlobalObject
    return nullptr;
  }

  RefPtr<UniFFIRustCallStatus> callStatus = new UniFFIRustCallStatus(global);
  return callStatus.forget();
}

JSObject* UniFFIRustCallStatus::WrapObject(JSContext* aCx,
                                           JS::Handle<JSObject*> aGivenProto) {
  return UniFFIRustCallStatus_Binding::Wrap(aCx, this, aGivenProto);
}

void UniFFIRustCallStatus::update(RustCallStatus& aStatus) {
  mCode = aStatus.code;
  if (aStatus.error_buf.data) {
    // Rust returned data in the error_buf
    mBuf = OwnedRustBuffer(aStatus.error_buf);
  } else {
    // Rust returned an empty error_buf, create an invalid OwnedRustBuffer for
    // it.
    mBuf = OwnedRustBuffer();
  }
}

void UniFFIRustCallStatus::GetArrayBuffer(JSContext* aCx,
                                          JS::MutableHandle<JSObject*> aResult,
                                          ErrorResult& aErrorResult) {
  if (mBuf.isValid()) {
    // We have data in mBuf, return it
    JS::RootedObject arrayBuf(aCx, mBuf.intoArrayBuffer(aCx));
    if (!arrayBuf) {
      aErrorResult.ThrowUnknownError("unable to allocate result ArrayBuffer");
      return;
    }
    aResult.set(arrayBuf);
  } else {
    // mBuf is invalid, return an empty array buffer
    JS::RootedObject arrayBuf(aCx, JS::NewArrayBuffer(aCx, 0));
    aResult.set(arrayBuf);
  }
}

}  // namespace mozilla::dom
