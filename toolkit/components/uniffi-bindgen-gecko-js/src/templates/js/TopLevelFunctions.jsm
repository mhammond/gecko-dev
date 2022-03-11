{%- for func in ci.iter_function_definitions() %}
function {{ func.js_name() }}({{ func.js_arg_names() }}) {
    const liftResult = (result) => {{ func.js_ffi_return_type() }}.lift(result)
    const liftError = null; // TODO
    const callResult = {{ ci.scaffolding_name() }}.{{func.ffi_func().js_name()}}(
        {%- for arg in func.arguments() -%}
        {{ arg.js_lower_fn_name() }}({{ arg.name() }}),
        {%- endfor %}
    )
    {%- if func.is_async() %}
    return callResult.then((result) => handleRustResult(result,  liftResult, liftError));
    {%- else %}
    return handleRustResult(callResult,  liftResult, liftError);
    {%- endif %}
}

EXPORTED_SYMBOLS.push("{{ func.js_name() }}");


{%- endfor %}
