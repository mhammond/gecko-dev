/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Represents the result of a call into Rust via UniFFI
dictionary UniFFIRustCallResult {
    // Result of the call, one of:
    //
    // CALL_SUCCESS = 0;        // Successful returns
    // CALL_ERROR = 1;          // Rust Err values
    // CALL_INTERNAL_ERROR = 2; // Internal/unexpected errors
    short code = 0;

    // For CALL_SUCCESS, this will be the return value
    // For CALL_ERROR, this will be an ArrayBuffer storing the serialized error value
    // For CALL_INTERNAL_ERROR, this will stay null
    any data = null;

    // For CALL_INTERNAL_ERROR only, this will be set to a message describing the error
    DOMString internalErrorMessage = "";
};
