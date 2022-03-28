{%- let string_type = Type::String %}
{%- let string_ffi_converter = string_type.ffi_converter() %}


class {{ ffi_converter }} extends FfiConverterArrayBuffer {
    static read(dataStream) {
        const len = dataStream.readUint32();
        const map = {};
        for (let i = 0; i < len; i++) {
            const key = {{ string_ffi_converter }}.read(dataStream);
            const value = {{ inner.ffi_converter() }}.read(dataStream);
            map[key] = value;
        }

        return map;
    }

    static write(dataStream, value) {
        dataStream.writeUint32(Object.keys(value).length);
        for (const key in value) {
            {{ string_ffi_converter }}.write(dataStream, key);
            {{ inner.ffi_converter() }}.write(dataStream, value[key]);
        }
    }

    static computeSize(value) {
        // The size of the length
        let size = 4;
        for (const key in value) {
            size += {{ string_ffi_converter }}.computeSize(key);
            size += {{ inner.ffi_converter() }}.computeSize(value[key]);
        }
        return size;
    }
}

