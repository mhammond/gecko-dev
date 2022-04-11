{%- let object = ci.get_object_definition(name).unwrap() %}

class {{ object.nm() }} {
    {%- match object.primary_constructor() %}
    {%- when Some with (cons) %}
    // Use `init` to instantiate this class.
    // DO NOT USE THIS CONSTRUCTOR DIRECTLY
    constructor(ptr) {
        if (!ptr) {
            throw new UniFFIError("Attempting to construct an object that needs to be constructed asynchronously" +
            "Please use the init async function")
        }
        this.ptr = ptr;
    }

    {%- if cons.is_async() %}
    /**
     * An async constructor for {{ object.nm() }}.
     * 
     * @returns {Promise<{{ object.nm() }}>}: A promise that resolves
     *      to a newly constructed {{ object.nm() }}
     */
    {%- else %}
    /**
     * A constructor for {{ object.nm() }}.
     * 
     * @returns { {{ object.nm() }} }
     */
    {%- endif %}
    static init({{cons.arg_names()}}) {
        {% call js::call_constructor(cons, type_) %}
    }
    {%- else %}
    {%- endmatch %}

    {%- for meth in object.methods() %}
    {{ meth.nm() }}({{ meth.arg_names() }}) {
        {% call js::call_method(meth, type_) %}
    }
    {%- endfor %}
}

class {{ ffi_converter }} extends FfiConverter {
    static lift(value) {
        return new {{ object.nm() }}(value);
    }

    static lower(value) {
        return value.ptr;
    }

    static read(dataStream) {
        return lift(dataStream.readInt64());
    }

    static write(dataStream, value) {
        dataStream.writeInt64(value.ptr);
    }

    static computeSize(value) {
        return 8;
    }
}

EXPORTED_SYMBOLS.push("{{ object.nm() }}");
