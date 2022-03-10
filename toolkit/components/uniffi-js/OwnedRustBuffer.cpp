/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim:set ts=2 sw=2 sts=2 et cindent: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "mozilla/dom/OwnedRustBuffer.h"

namespace mozilla::dom {

OwnedRustBuffer::OwnedRustBuffer(const RustBuffer& aBuf) {
  mBuf = aBuf;
  MOZ_ASSERT(isValid());
}

OwnedRustBuffer::OwnedRustBuffer(const ArrayBuffer& aArrayBuffer,
                                 ErrorResult& aRv) {
  if (aArrayBuffer.Length() > INT32_MAX) {
    mBuf = { 0 };
    aRv.ThrowRangeError("Input ArrayBuffer is too large");
    return;
  }

  RustCallStatus status{};
  RustBuffer buf = uniffi_rustbuffer_alloc(
      static_cast<int32_t>(aArrayBuffer.Length()), &status);
  buf.len = aArrayBuffer.Length();
  if (status.code != 0) {
    if (status.error_buf.data) {
      aRv.ThrowUnknownError(
          nsDependentCSubstring(reinterpret_cast<char*>(status.error_buf.data),
                                status.error_buf.len));

      RustCallStatus status2{};
      uniffi_rustbuffer_free(status.error_buf, &status2);
      // Don't check the status of free, it shouldn't fail and if it does
      // there's nothing we can do at this point.

    } else {
      aRv.ThrowUnknownError("Unknown error allocating rust buffer");
    }
    mBuf = { 0 };
    return;
  }

  memcpy(buf.data, aArrayBuffer.Data(), buf.len);
  mBuf = buf;
  MOZ_ASSERT(isValid());
}

OwnedRustBuffer::OwnedRustBuffer(OwnedRustBuffer&& aOther) : mBuf(aOther.mBuf) {
  aOther.mBuf = RustBuffer{0};
}

OwnedRustBuffer& OwnedRustBuffer::operator=(OwnedRustBuffer&& aOther) {
  mBuf = aOther.mBuf;
  aOther.mBuf = RustBuffer{0};
  return *this;
}

OwnedRustBuffer::~OwnedRustBuffer() {
  if (isValid()) {
    RustCallStatus status{};
    uniffi_rustbuffer_free(mBuf, &status);
    if (status.code != 0) {
      MOZ_CRASH("Freeing a RustBuffer should never fail");
    }
  }
}

RustBuffer OwnedRustBuffer::intoRustBuffer() {
  RustBuffer rv = mBuf;
  mBuf = {};
  MOZ_ASSERT(wasMoved());
  return rv;
}

JSObject* OwnedRustBuffer::intoArrayBuffer(JSContext* cx) {
  int32_t len = mBuf.len;
  void* data = mBuf.data;
  auto userData = MakeUnique<OwnedRustBuffer>(std::move(*this));
  MOZ_ASSERT(wasMoved());
  return JS::NewExternalArrayBuffer(cx, len, data, &ArrayBufferFreeFunc,
                                    userData.release());
}

void OwnedRustBuffer::ArrayBufferFreeFunc(void* contents, void* userData) {
  UniquePtr<OwnedRustBuffer> buf{reinterpret_cast<OwnedRustBuffer*>(userData)};
}
}  // namespace mozilla::dom
