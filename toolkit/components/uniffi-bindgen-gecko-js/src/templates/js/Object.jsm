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
    static init({{cons.arg_names()}}) {
        const liftResult = (resultPtr) => {
            return new {{ object.nm() }}(resultPtr);
        };
        {%- match cons.throws_type() %}
        {%- when Some with (err_type) %}
        const liftError = (data) => {{ err_type.ffi_converter() }}.lift(data);
        {%- else %}
        const liftError = null;
        {%- endmatch %}
    
        const callResult = {{ ci.scaffolding_name() }}.{{cons.ffi_func().nm()}}(
            {%- for arg in cons.arguments() -%}
            {{ arg.lower_fn_name() }}({{ arg.name() }}),
            {%- endfor %}
        )
        {%- if cons.is_async() %}
        return callResult.then((result) => handleRustResult(result,  liftResult, liftError));
        {%- else %}
        return handleRustResult(callResult,  liftResult, liftError);
        {%- endif %}
    }
    {%- else %}
    
    /**
     * A constructor for {{ object.nm() }}.
     * 
     * @returns { {{ object.nm() }} }
     */
    static init({{ cons.arg_names() }}) {
        const liftResult = (resultPtr) => {
            return new {{ object.nm() }}(resultPtr);
        };
        {%- match cons.throws_type() %}
        {%- when Some with (err_type) %}
        const liftError = (data) => {{ err_type.ffi_converter() }}.lift(data);
        {%- else %}
        const liftError = null;
        {%- endmatch %}
    
        const callResult = {{ ci.scaffolding_name() }}.{{cons.ffi_func().nm()}}(
            {%- for arg in cons.arguments() -%}
            {{ arg.lower_fn_name() }}({{ arg.name() }}),
            {%- endfor %}
        )
        return handleRustResult(callResult,  liftResult, liftError);
    }
    {%- endif %}
    {%- else %}
    {%- endmatch %}

    {%- for meth in object.methods() %}
    {{ meth.nm() }}({{ meth.arg_names() }}) {
        const liftResult = (result) => {{ meth.ffi_return_type() }}.lift(result);
        {%- match meth.throws_type() %}
        {%- when Some with (err_type) %}
        const liftError = (data) => {{ err_type.ffi_converter() }}.lift(data);
        {%- else %}
        const liftError = null;
        {%- endmatch %}
    
        const callResult = {{ ci.scaffolding_name() }}.{{meth.ffi_func().nm()}}(this.ptr,
            {%- for arg in meth.arguments() -%}
            {{ arg.lower_fn_name() }}({{ arg.name() }}),
            {%- endfor %}
        )
        {%- if meth.is_async() %}
        return callResult.then((result) => handleRustResult(result,  liftResult, liftError));
        {%- else %}
        return handleRustResult(callResult,  liftResult, liftError);
        {%- endif %}
    }
    {%- endfor %}
}

class {{ ffi_converter }} {
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
