class {{ ffi_converter }} extends FfiConverterArrayBuffer {
    static read(dataStream) {
        const code = dataStream.readUint8(0);
        switch (code) {
            case 0:
                return null
            case 1:
                return {{ inner.ffi_converter() }}.read(dataStream)
            default:
                throw UniFFIError(`Unexpected code: ${code}`);
        }
    }

    static write(dataStream, value) {
        if (!value) {
            dataStream.writeUint8(0);
        }
        dataStream.writeUint8(1);
        {{ inner.ffi_converter() }}.write(dataStream, value)
    }

    static computeSize() {
        return 1 + {{ inner.ffi_converter() }}.computeSize()
    }
}

