{%- for func in ci.iter_function_definitions() %}
function {{ func.nm() }}({{ func.arg_names() }}) {
    const liftResult = (result) => {{ func.ffi_return_type() }}.lift(result);
    {%- match func.throws_type() %}
    {%- when Some with (err_type) %}
    const liftError = (data) => {{ err_type.ffi_converter() }}.lift(data); // TODO
    {%- else %}
    const liftError = null;
    {%- endmatch %}

    const callResult = {{ ci.scaffolding_name() }}.{{func.ffi_func().nm()}}(
        {%- for arg in func.arguments() -%}
        {{ arg.lower_fn_name() }}({{ arg.name() }}),
        {%- endfor %}
    )
    {%- if func.is_async() %}
    return callResult.then((result) => handleRustResult(result,  liftResult, liftError));
    {%- else %}
    return handleRustResult(callResult,  liftResult, liftError);
    {%- endif %}
}

EXPORTED_SYMBOLS.push("{{ func.nm() }}");


{%- endfor %}
