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

class FfiConverterDouble {
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
  
