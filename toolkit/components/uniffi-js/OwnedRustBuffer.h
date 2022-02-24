/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim:set ts=2 sw=2 sts=2 et cindent: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_dom_OwnedRustBuffer_h
#define mozilla_dom_OwnedRustBuffer_h

#include "mozilla/ErrorResult.h"
#include "mozilla/dom/TypedArray.h"
#include "UniFFI.h"

namespace mozilla::dom {

// RustBuffer that's owned by the JS code and handles the memory management
class OwnedRustBuffer final {
 private:
  RustBuffer mBuf;

 public:
  // The default constructor creates an invalid OwnedRustBuffer
  OwnedRustBuffer() = default;

  // Constructor for creating an OwnedRustBuffer from a raw RustBuffer struct
  // that was returned by Rust (therefore we now own the RustBuffer).
  OwnedRustBuffer(const RustBuffer& aBuf);

  // Constructor for creating an OwnedRustBuffer from an ArrayBuffer. Will set
  // aRv to failed and construct an invalid OwnedRustBuffer if the conversion
  // failed.
  OwnedRustBuffer(const ArrayBuffer& aArrayBuffer, ErrorResult& aRv);

  // Manual implementation of move constructor and assignment operator.
  OwnedRustBuffer(OwnedRustBuffer&& aOther);
  OwnedRustBuffer& operator=(OwnedRustBuffer&& aOther);

  // Delete copy & move constructor as this type is non-copyable.
  OwnedRustBuffer(const OwnedRustBuffer&) = delete;
  OwnedRustBuffer& operator=(const OwnedRustBuffer&) = delete;

  // Destructor that frees the RustBuffer if it is still valid
  ~OwnedRustBuffer();

  // Moves the buffer out of this `OwnedArrayBuffer` into a raw `RustBuffer`
  // struct.  The raw struct must be passed into a Rust function, transfering
  // ownership to Rust.  After this call the buffer will no longer be valid.
  RustBuffer intoRustBuffer();

  // Moves the buffer out of this `OwnedArrayBuffer` into a JS ArrayBuffer.
  // This transfers ownership into the JS engine.  After this call the buffer
  // will no longer be valid.
  JSObject* intoArrayBuffer(JSContext* cx);

  // Is this RustBuffer pointing to valid data?
  bool isValid() const { return mBuf.data != nullptr; }

 private:
  // Helper function used by IntoArrayBuffer.
  static void ArrayBufferFreeFunc(void* contents, void* userData);
};

}  // namespace mozilla::dom

#endif  // mozilla_dom_OwnedRustBuffer_h
