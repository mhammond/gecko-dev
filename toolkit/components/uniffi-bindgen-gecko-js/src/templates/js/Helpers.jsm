// Write/Read data to/from an ArrayBuffer
class ArrayBufferDataStream {
    constructor(arrayBuffer) {
      this.dataView = new DataView(arrayBuffer);
      this.pos = 0;
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
  
    readUint8() {
      let rv = this.dataView.getUint8(this.pos);
      this.pos += 1;
      return rv;
    }
  
  
    // TODO: write more methods
  }


  function makeRustCall(callback, liftErrCallback) {
    let status = new UniFFIRustCallStatus();
    let rv = callback(status);
    switch (status.code) {
      case UniFFIRustCallStatus.CALL_SUCCESS:
        return rv;
  
      case UniFFIRustCallStatus.CALL_ERROR:
        throw liftErrCallback(status.getArrayBuffer());
  
      case UniFFIRustCallStatus.CALL_PANIC:
        // Theoretical code to handle Rust panics.  It's theoretical at
        // this point because gecko sets the panic=abort flag for rustc,
        // which prevents the UniFFI panic catching code from running.
  
        // Try to get the panic message
        let errorBuf = status.getArrayBuffer();
        if (errorBuf.byteLength > 0) {
          throw UniFFIPanic(liftString(errorBuf));
        } else {
          throw UniFFIPanic("Unknown panic in Rust code");
        }
  
      default:
        throw UniFFIError(`Unexpected status code: ${status.code}`);
    }
  }
  
  class UniFFIError {
    constructor(message) {
      this.message = message;
    }
  }
  class UniFFIPanic extends UniFFIError {}

class FFIConverterDouble {
    static lift(value) {
        return value;
    }
    static lower(value) {
        return value;
    }
    static computeSize() {
        return 8;
    }
    static write(dataStream, double) {
        dataStream.writeFloat64(double)
    }

    static read(dataStream) {
        return dataStream.readFloat64()
    }
}
  