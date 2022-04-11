class {{ ffi_converter }} extends FfiConverter {
    static computeSize() {
        return 2;
    }
    static lift(value) {
        return value;
    }
    static lower(value) {
        return value;
    }
    static write(dataStream, value) {
        dataStream.writeInt16(value)
    }
    static read(dataStream) {
        return dataStream.readInt16()
    }
}
