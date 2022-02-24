/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim:set ts=2 sw=2 sts=2 et cindent: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_dom_UniFFIRustCallStatus_h
#define mozilla_dom_UniFFIRustCallStatus_h

#include "nsCOMPtr.h"
#include "nsIGlobalObject.h"
#include "OwnedRustBuffer.h"
#include "UniFFI.h"

namespace mozilla {
namespace dom {

class GlobalObject;

}  // namespace dom
}  // namespace mozilla

namespace mozilla::dom {

class UniFFIRustCallStatus final
    : public nsISupports /* or NonRefcountedDOMObject if this is a
                            non-refcounted object */
    ,
      public nsWrapperCache /* Change wrapperCache in the binding configuration
                               if you don't want this */
{
 public:
  NS_DECL_CYCLE_COLLECTING_ISUPPORTS
  NS_DECL_CYCLE_COLLECTION_SCRIPT_HOLDER_CLASS(UniFFIRustCallStatus)

 public:
  // Constructor that initializes the status to CALL_SUCCESS
  UniFFIRustCallStatus(nsCOMPtr<nsIGlobalObject> aGlobal);

  // Update this object after a rust call
  void update(RustCallStatus& aStatus);

 private:
  ~UniFFIRustCallStatus() = default;

  nsCOMPtr<nsIGlobalObject> mGlobal;
  int8_t mCode;
  OwnedRustBuffer mBuf;

 public:
  static already_AddRefed<UniFFIRustCallStatus> Constructor(
      const GlobalObject& aGlobal);
  JSObject* WrapObject(JSContext* aCx,
                       JS::Handle<JSObject*> aGivenProto) override;
  nsIGlobalObject* GetParentObject() const { return mGlobal; }
  int16_t Code() { return mCode; }
  void GetArrayBuffer(JSContext* aCx, JS::MutableHandle<JSObject*> aResult,
                      ErrorResult& aErrorResult);
};

}  // namespace mozilla::dom

#endif  // mozilla_dom_UniFFIRustCallStatus_h
