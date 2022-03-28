

class FfiConverterString {
    static lift(buf) {
        const decoder = new TextDecoder();
        const utf8Arr = new Uint8Array(buf);
        return decoder.decode(utf8Arr);
    }
    static lower(value) {
        const encoder = new TextEncoder();
        return encoder.encode(value).buffer;
    }

    static write(dataStream, value) {
        const encoder = new TextEncoder();
        const utf8Arr = encoder.encode(value);
        dataStream.writeUint32(utf8Arr.length);
        dataStream.writeUint8Array(utf8Arr);
    }

    static read(dataStream) {
        const decoder = new TextDecoder();
        const size = dataStream.readUint32();
        const utf8Arr = dataStream.readUint8Array(size);
        return decoder.decode(utf8Arr);
    }

    static computeSize(value) {
        const encoder = new TextEncoder();
        return 4 + encoder.encode(value).length
    }
}
  
