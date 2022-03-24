class {{ ffi_converter }} {
    static computeSize() {
        return 8;
    }
    static lift(value) {
        return value;
    }
    static lower(value) {
        return value;
    }
    static write(dataStream, value) {
        dataStream.writeUint64(value)
    }
    static read(dataStream) {
        return dataStream.readUint64()
    }
}
