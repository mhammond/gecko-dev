// Hand-written JS targeted at the Geometry app.

"use strict";

var EXPORTED_SYMBOLS = ["gradient", "intersection", "Point", "Line"];

class Point {
  constructor(coord_x, coord_y) {
     this.coord_x = coord_x;
     this.coord_y = coord_y;
  }
}

class Line {
  constructor(start, end) {
     this.start = start;
     this.end = end;
  }
}

function gradient(ln) {
  return FFIConverterDouble.lift(
    makeRustCall((status) => GeometryScaffolding.geometryEb69Gradient(FFIConverterLine.lower(ln), status))
  );
}

function intersection(ln1, ln2) {
  return FFIConverterOptionalPoint.lift(
    makeRustCall((status) => GeometryScaffolding.geometryEb69Intersection(FFIConverterLine.lower(ln1), FFIConverterLine.lower(ln2), status))
  );
}

function makeRustCall(callback) {
  let status = new UniFFIRustCallStatus();
  let rv = callback(status);
  switch (status.code) {
    case UniFFIRustCallStatus.CALL_SUCCESS:
      return rv;

    case UniFFIRustCallStatus.CALL_ERROR:
      throw liftCalcError(status.getArrayBuffer());

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

let FFIConverterDouble = {
  lift: (value) => value,
  lower: (value) => value,
}

let FFIConverterLine = {
  lift: function(buf) {
    let dataStream = new ArrayBufferDataStream(buf);
    return new Line(
      FFIConverterPoint.read(dataStream),
      FFIConverterPoint.read(dataStream),
    );
  },
  lower: function(line) {
      // TODO: calculate the array size
      let buf = new ArrayBuffer(32);

      let dataStream = new ArrayBufferDataStream(buf);
      FFIConverterPoint.write(dataStream, line.start);
      FFIConverterPoint.write(dataStream, line.end);
      return buf;
  }
}

let FFIConverterOptionalPoint = {
  lift: function(buf) {
    let dataStream = new ArrayBufferDataStream(buf);
    let code = dataStream.readUint8(0);
    switch (code) {
        case 0:
            return null;
        case 1:
            return FFIConverterPoint.read(dataStream)
        default:
          throw UniFFIError(`Unexpected code: ${code}`);
    }
  },
}

let FFIConverterPoint = {
  lift: function(buf) {
    return this.read(new ArrayBufferDataStream(buf));
  },
  lower: function(point) {
    // TODO: calculate the array size
    let buf = new ArrayBuffer(16);
    let dataStream = new ArrayBufferDataStream(buf);
    this.write(dataStream, point);
    return buf;
  },
  read: function(dataStream) {
    return new Point(
        dataStream.readFloat64(),
        dataStream.readFloat64(),
    );
  },
  write: function(dataStream, point) {
    dataStream.writeFloat64(point.coord_x);
    dataStream.writeFloat64(point.coord_y);
  },
}