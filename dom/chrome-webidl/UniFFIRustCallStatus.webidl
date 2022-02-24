/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Represents the result of a call into Rust via UniFFI
[ChromeOnly, Exposed=Window]
interface UniFFIRustCallStatus {
    constructor();

    const short CALL_SUCCESS = 0;
    const short CALL_ERROR = 1;
    const short CALL_PANIC = 2;

    // One of the CALL_* constants
    readonly attribute short code;

    // ArrayBuffer with extra details for CALL_ERROR/CALL_PANIC
    //   - For CALL_ERROR, this will contain the error type serialized into a buffer
    //   - For CALL_PANIC, this may contain the error message as a UTF-8
    //     string.  However, getting the error message is not guaranteed and
    //     this may be an empty buffer.
    //
    // Getting the array buffer transfers ownership.  Subsequent calls will return an empty array buffer.
    [Throws] ArrayBuffer getArrayBuffer();
};
