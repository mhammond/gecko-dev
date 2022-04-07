// UniFFIRustCallResult.code values
const CALL_SUCCESS = 0;
const CALL_ERROR = 1;
const CALL_INTERNAL_ERROR = 2;

// Write/Read data to/from an ArrayBuffer
class ArrayBufferDataStream {
    constructor(arrayBuffer) {
        this.dataView = new DataView(arrayBuffer);
        this.pos = 0;
    }

    readUint8() {
        let rv = this.dataView.getUint8(this.pos);
        this.pos += 1;
        return rv;
    }

    readUint16() {
        let rv = this.dataView.getUint16(this.pos);
        this.pos += 2;
        return rv;
    }

    readUint32() {
        let rv = this.dataView.getUint32(this.pos);
        this.pos += 4;
        return rv;
    }

    readUint64() {
        let rv = this.dataView.getUint64(this.pos);
        this.pos += 8;
        return rv;
    }

    readInt8() {
        let rv = this.dataView.getInt8(this.pos);
        this.pos += 1;
        return rv;
    }

    readInt16() {
        let rv = this.dataView.getInt16(this.pos);
        this.pos += 2;
        return rv;
    }

    readInt32() {
        let rv = this.dataView.getInt32(this.pos);
        this.pos += 4;
        return rv;
    }

    readInt64() {
        let rv = this.dataView.getInt64(this.pos);
        this.pos += 8;
        return rv;
    }

    readFloat32() {
        let rv = this.dataView.getFloat32(this.pos);
        this.pos += 4;
        return rv;
    }

    writeFloat32(value) {
        this.dataView.setFloat32(this.pos, value);
        this.pos += 4;
    }

    readFloat64() {
        let rv = this.dataView.getFloat64(this.pos);
        this.pos += 8;
        return rv;
    }

    writeFloat64(value) {
        this.dataView.setFloat64(this.pos, value);
        this.pos += 8;
    }

    writeUint8(value) {
      this.dataView.setUint8(this.pos, value);
      this.pos += 1;
    }

    writeUint32(value) {
      this.dataView.setUint32(this.pos, value);
      this.pos += 4;
    }

    writeString(value) {
      const encoder = new TextEncoder();
      // Note: in order to efficiently write this data, we first write the
      // string data, reserving 4 bytes for the size.
      const dest = new Uint8Array(this.dataView.buffer, this.pos + 4);
      const encodeResult = encoder.encodeInto(value, dest);
      if (encodeResult.read != value.length) {
        throw new UniFFIError(
            "writeString: out of space when writing to ArrayBuffer.  Did the computeSize() method returned the wrong result?"
        );
      }
      const size = encodeResult.written;
      // Next, go back and write the size before the string data
      this.dataView.setUint32(this.pos, size);
      // Finally, advance our position past both the size and string data
      this.pos += size + 4;
    }

    writeInt32(value) {
        this.dataView.setInt32(this.pos, value);
        this.pos += 4;
    }

    readString() {
      const decoder = new TextDecoder();
      const size = this.readUint32();
      const source = new Uint8Array(this.dataView.buffer, this.pos, size)
      const value = decoder.decode(source);
      this.pos += size;
      return value;
    }
}

function handleRustResult(result, liftCallback, liftErrCallback) {
    switch (result.code) {
        case CALL_SUCCESS:
            return liftCallback(result.data);

        case CALL_ERROR:
            throw liftErrCallback(result.data);

        case CALL_INTERNAL_ERROR:
            let message = result.internalErrorMessage;
            if (message) {
                throw new UniFFIInternalError(message);
            } else {
                throw new UniFFIInternalError("Unknown error");
            }

        default:
            throw new UniFFIError(`Unexpected status code: ${result.code}`);
    }
}

class UniFFIError {
    constructor(message) {
        this.message = message;
    }
}

class UniFFIInternalError extends UniFFIError {}

// Base class for FFI converters that lift/lower by reading/writing to an ArrayBuffer
class FfiConverterArrayBuffer {
    static lift(buf) {
        return this.read(new ArrayBufferDataStream(buf));
    }

    static lower(value) {
        const buf = new ArrayBuffer(this.computeSize(value));
        const dataStream = new ArrayBufferDataStream(buf);
        this.write(dataStream, value);
        return buf;
    }
}
