
{%- for record in ci.iter_record_definitions() %}

class {{ record.js_name() }} {
    constructor({{ record.js_constructor_field_list() }}) {
        {%- for field in record.fields() %}
        this.{{field.js_name()}} = {{ field.js_name() }};
        {%- endfor %}
    }
}

class FfiConverter{{ record.js_name() }} {
    static lift(buf) {
        return this.read(new ArrayBufferDataStream(buf));
    }
    static lower({{ record.js_var_name() }}) {
        let buf = new ArrayBuffer(this.computeSize());
        let dataStream = new ArrayBufferDataStream(buf);
        this.write(dataStream, {{ record.js_var_name() }});
    }
    static read(dataStream) {
        return new {{record.js_name()}}(
            {%- for field in record.fields() %}
            {{ field.read_datastream_fn() }}(dataStream)
           {%- if !loop.last %}, {% endif %}
           {%- endfor %}
        );
    }
    static write(dataStream, {{ record.js_var_name() }}) {
        {%- for field in record.fields() %}
        {{ field.write_datastream_fn() }}(dataStream, {{ record.js_var_name() }}.{{field.js_name()}});
        {%- endfor %}
    }

    static computeSize() {
        const totalSize = 0;
        {%- for field in record.fields() %}
        totalSize += {{ field.js_ffi_converter() }}.computeSize();
        {%- endfor %}
        return totalSize
    }
}

{%- endfor %}