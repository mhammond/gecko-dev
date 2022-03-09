{%- for func in ci.iter_function_definitions() %}
function {{ func.js_name() }}({{ func.js_arg_names() }}) {
    return {{ func.js_ffi_return_type() }}.lift(
        makeRustCall((status) => {{ ci.scaffolding_name() }}.{{func.ffi_func().js_name()}}({%- for arg in func.arguments() -%}
            {{ arg.js_lift_fn_name() }}({{ arg.name() }}),
            {%- endfor %}
            status
        )
    )
}


{%- endfor %}