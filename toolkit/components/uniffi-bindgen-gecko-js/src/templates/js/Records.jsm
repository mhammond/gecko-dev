
{%- for record in ci.iter_record_definitions() %}

class {{ record.nm() }} {
    constructor({{ record.constructor_field_list() }}) {
        {%- for field in record.fields() %}
        this.{{field.nm()}} = {{ field.nm() }};
        {%- endfor %}
    }
}

class FfiConverter{{ record.nm() }} {
    static lift(buf) {
        return this.read(new ArrayBufferDataStream(buf));
    }
    static lower(value) {
        const buf = new ArrayBuffer(this.computeSize());
        const dataStream = new ArrayBufferDataStream(buf);
        this.write(dataStream, value);
        return buf;
    }
    static read(dataStream) {
        return new {{record.nm()}}(
            {%- for field in record.fields() %}
            {{ field.read_datastream_fn() }}(dataStream)
           {%- if !loop.last %}, {% endif %}
           {%- endfor %}
        );
    }
    static write(dataStream, value) {
        {%- for field in record.fields() %}
        {{ field.write_datastream_fn() }}(dataStream, value.{{field.nm()}});
        {%- endfor %}
    }

    static computeSize() {
        let totalSize = 0;
        {%- for field in record.fields() %}
        totalSize += {{ field.ffi_converter() }}.computeSize();
        {%- endfor %}
        return totalSize
    }
}

class FfiConverterOptional{{ record.nm() }} {
    static lift(buf) {
        return this.read(new ArrayBufferDataStream(buf));
    }

    static lower(value) {
        const buf = new ArrayBuffer(this.computeSize());
        const dataStream = new ArrayBufferDataStream(buf);
        this.write(dataStream, value);
        return buf;
    }

    static read(dataStream) {
        const code = dataStream.readUint8(0);
        switch (code) {
            case 0:
                return null
            case 1:
                return FfiConverter{{record.nm()}}.read(dataStream);
            default:
                throw UniFFIError(`Unexpected code: ${code}`);
        }
    }

    static write(dataStream, value) {
        if (!value) {
            dataStream.writeUint8(0);
            return buf;
        }
        dataStream.writeUint8(1);
        FfiConverter{{record.nm()}}.write(dataStream, value);
        return buf;
    }

    static computeSize() {
        return 1 + FfiConverter{{record.nm()}}.computeSize();
    }
}

EXPORTED_SYMBOLS.push("{{ record.nm() }}");

{%- endfor %}
