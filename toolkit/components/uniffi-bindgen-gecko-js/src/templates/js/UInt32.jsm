class {{ ffi_converter }} extends FfiConverter {
    static computeSize() {
        return 4;
    }
    static lift(value) {
        return value;
    }
    static lower(value) {
        return value;
    }
    static write(dataStream, value) {
        dataStream.writeUint32(value)
    }
    static read(dataStream) {
        return dataStream.readUint32()
    }
}
