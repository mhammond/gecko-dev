class {{ ffi_converter }} extends FfiConverter {
    static computeSize() {
        return 1;
    }
    static lift(value) {
        return value;
    }
    static lower(value) {
        return value;
    }
    static write(dataStream, value) {
        dataStream.writeUint8(value)
    }
    static read(dataStream) {
        return dataStream.readUint8()
    }
}
