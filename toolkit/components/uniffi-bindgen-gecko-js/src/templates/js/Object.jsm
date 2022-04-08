{%- let object = ci.get_object_definition(name).unwrap() %}

class {{ object.nm() }} {
    {%- match object.primary_constructor() %}
    {%- when Some with (cons) %}
    constructor(ptr) {
        if (!ptr) {
            throw new UniFFIError("Attempting to construct an object that needs to be constructed asynchronously" +
            "Please use the init async function")
        }
        this.ptr = ptr;
    }
    
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

EXPORTED_SYMBOLS.push("{{ object.nm() }}");
