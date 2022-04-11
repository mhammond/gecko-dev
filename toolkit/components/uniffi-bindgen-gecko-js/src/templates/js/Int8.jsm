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
        dataStream.writeInt8(value)
    }
    static read(dataStream) {
        return dataStream.readInt8()
    }
}
