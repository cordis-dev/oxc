//! All registered lint rules.
//!
//! New rules need be added to these `mod` statements and also the macro at the bottom.
//!
//! These modules are declared manually because `cargo fmt` stops formatting these files with they
//! are inside a proc macro.

/// <https://github.com/import-js/eslint-plugin-import>
mod import {
    pub mod consistent_type_specifier_style;
    pub mod default;
    pub mod export;
    pub mod exports_last;
    pub mod extensions;
    pub mod first;
    pub mod group_exports;
    pub mod max_dependencies;
    pub mod named;
    pub mod namespace;
    pub mod no_absolute_path;
    pub mod no_amd;
    pub mod no_anonymous_default_export;
    pub mod no_commonjs;
    pub mod no_cycle;
    pub mod no_default_export;
    pub mod no_duplicates;
    pub mod no_dynamic_require;
    pub mod no_empty_named_blocks;
    pub mod no_mutable_exports;
    pub mod no_named_as_default;
    pub mod no_named_as_default_member;
    pub mod no_named_default;
    pub mod no_namespace;
    pub mod no_self_import;
    pub mod no_unassigned_import;
    pub mod no_webpack_loader_syntax;
    pub mod prefer_default_export;
    pub mod unambiguous;
}

mod eslint {
    pub mod array_callback_return;
    pub mod arrow_body_style;
    pub mod block_scoped_var;
    pub mod curly;
    pub mod default_case;
    pub mod default_case_last;
    pub mod default_param_last;
    pub mod eqeqeq;
    pub mod complexity;
    pub mod no_warning_comments;
    pub mod for_direction;
    pub mod func_names;
    pub mod func_style;
    pub mod getter_return;
    pub mod grouped_accessor_pairs;
    pub mod guard_for_in;
    pub mod id_length;
    pub mod init_declarations;
    pub mod max_classes_per_file;
    pub mod max_depth;
    pub mod max_lines;
    pub mod max_lines_per_function;
    pub mod max_nested_callbacks;
    pub mod max_params;
    pub mod new_cap;
    pub mod no_alert;
    pub mod no_array_constructor;
    pub mod no_async_promise_executor;
    pub mod no_await_in_loop;
    pub mod no_bitwise;
    pub mod no_caller;
    pub mod no_case_declarations;
    pub mod no_class_assign;
    pub mod no_compare_neg_zero;
    pub mod no_cond_assign;
    pub mod no_console;
    pub mod no_const_assign;
    pub mod no_constant_binary_expression;
    pub mod no_constant_condition;
    pub mod no_constructor_return;
    pub mod no_continue;
    pub mod no_control_regex;
    pub mod no_debugger;
    pub mod no_delete_var;
    pub mod no_div_regex;
    pub mod no_dupe_class_members;
    pub mod no_dupe_else_if;
    pub mod no_dupe_keys;
    pub mod no_duplicate_case;
    pub mod no_duplicate_imports;
    pub mod no_else_return;
    pub mod no_empty;
    pub mod no_empty_character_class;
    pub mod no_empty_function;
    pub mod no_empty_pattern;
    pub mod no_empty_static_block;
    pub mod no_eq_null;
    pub mod no_eval;
    pub mod no_ex_assign;
    pub mod no_extend_native;
    pub mod no_extra_bind;
    pub mod no_extra_boolean_cast;
    pub mod no_extra_label;
    pub mod no_fallthrough;
    pub mod no_func_assign;
    pub mod no_global_assign;
    pub mod no_import_assign;
    pub mod no_inner_declarations;
    pub mod no_invalid_regexp;
    pub mod no_irregular_whitespace;
    pub mod no_iterator;
    pub mod no_label_var;
    pub mod no_labels;
    pub mod no_lone_blocks;
    pub mod no_lonely_if;
    pub mod no_loss_of_precision;
    pub mod no_magic_numbers;
    pub mod no_multi_assign;
    pub mod no_multi_str;
    pub mod no_negated_condition;
    pub mod no_nested_ternary;
    pub mod no_new;
    pub mod no_new_func;
    pub mod no_new_native_nonconstructor;
    pub mod no_new_wrappers;
    pub mod no_nonoctal_decimal_escape;
    pub mod no_obj_calls;
    pub mod no_object_constructor;
    pub mod no_plusplus;
    pub mod no_proto;
    pub mod no_prototype_builtins;
    pub mod no_redeclare;
    pub mod no_regex_spaces;
    pub mod no_restricted_globals;
    pub mod no_restricted_imports;
    pub mod no_return_assign;
    pub mod no_script_url;
    pub mod no_self_assign;
    pub mod no_self_compare;
    pub mod no_setter_return;
    pub mod no_shadow_restricted_names;
    pub mod no_sparse_arrays;
    pub mod no_template_curly_in_string;
    pub mod no_ternary;
    pub mod no_this_before_super;
    pub mod no_throw_literal;
    pub mod no_undef;
    pub mod no_undefined;
    pub mod no_unexpected_multiline;
    pub mod no_unneeded_ternary;
    pub mod no_unreachable;
    pub mod no_unsafe_finally;
    pub mod no_unsafe_negation;
    pub mod no_unsafe_optional_chaining;
    pub mod no_unused_expressions;
    pub mod no_unused_labels;
    pub mod no_unused_private_class_members;
    pub mod no_unused_vars;
    pub mod no_useless_backreference;
    pub mod no_useless_call;
    pub mod no_useless_catch;
    pub mod no_useless_concat;
    pub mod no_useless_constructor;
    pub mod no_useless_escape;
    pub mod no_useless_rename;
    pub mod no_var;
    pub mod no_void;
    pub mod no_with;
    pub mod operator_assignment;
    pub mod prefer_exponentiation_operator;
    pub mod prefer_numeric_literals;
    pub mod prefer_object_has_own;
    pub mod prefer_object_spread;
    pub mod prefer_promise_reject_errors;
    pub mod prefer_rest_params;
    pub mod prefer_spread;
    pub mod radix;
    pub mod require_await;
    pub mod require_yield;
    pub mod sort_imports;
    pub mod sort_keys;
    pub mod sort_vars;
    pub mod symbol_description;
    pub mod unicode_bom;
    pub mod use_isnan;
    pub mod valid_typeof;
    pub mod vars_on_top;
    pub mod yoda;
}

mod typescript {
    pub mod adjacent_overload_signatures;
    pub mod array_type;
    pub mod ban_ts_comment;
    pub mod ban_tslint_comment;
    pub mod ban_types;
    pub mod consistent_generic_constructors;
    pub mod consistent_indexed_object_style;
    pub mod consistent_type_definitions;
    pub mod consistent_type_imports;
    pub mod explicit_function_return_type;
    pub mod explicit_module_boundary_types;
    pub mod no_confusing_non_null_assertion;
    pub mod no_duplicate_enum_values;
    pub mod no_dynamic_delete;
    pub mod no_empty_interface;
    pub mod no_empty_object_type;
    pub mod no_explicit_any;
    pub mod no_extra_non_null_assertion;
    pub mod no_extraneous_class;
    pub mod no_import_type_side_effects;
    pub mod no_inferrable_types;
    pub mod no_misused_new;
    pub mod no_namespace;
    pub mod no_non_null_asserted_nullish_coalescing;
    pub mod no_non_null_asserted_optional_chain;
    pub mod no_non_null_assertion;
    pub mod no_require_imports;
    pub mod no_this_alias;
    pub mod no_unnecessary_parameter_property_assignment;
    pub mod no_unnecessary_type_constraint;
    pub mod no_unsafe_declaration_merging;
    pub mod no_unsafe_function_type;

    pub mod no_useless_empty_export;
    pub mod no_var_requires;
    pub mod no_wrapper_object_types;
    pub mod prefer_as_const;
    pub mod prefer_enum_initializers;
    pub mod prefer_for_of;
    pub mod prefer_function_type;
    pub mod prefer_literal_enum_member;
    pub mod prefer_namespace_keyword;
    pub mod prefer_ts_expect_error;
    pub mod triple_slash_reference;
}

mod jest {
    pub mod consistent_test_it;
    pub mod expect_expect;
    pub mod max_expects;
    pub mod max_nested_describe;
    pub mod no_alias_methods;
    pub mod no_commented_out_tests;
    pub mod no_conditional_expect;
    pub mod no_conditional_in_test;
    pub mod no_confusing_set_timeout;
    pub mod no_deprecated_functions;
    pub mod no_disabled_tests;
    pub mod no_done_callback;
    pub mod no_duplicate_hooks;
    pub mod no_export;
    pub mod no_focused_tests;
    pub mod no_hooks;
    pub mod no_identical_title;
    pub mod no_interpolation_in_snapshots;
    pub mod no_jasmine_globals;
    pub mod no_large_snapshots;
    pub mod no_mocks_import;
    pub mod no_restricted_jest_methods;
    pub mod no_restricted_matchers;
    pub mod no_standalone_expect;
    pub mod no_test_prefixes;
    pub mod no_test_return_statement;
    pub mod no_untyped_mock_factory;
    pub mod prefer_called_with;
    pub mod prefer_comparison_matcher;
    pub mod prefer_each;
    pub mod prefer_equality_matcher;
    pub mod prefer_expect_resolves;
    pub mod prefer_hooks_in_order;
    pub mod prefer_hooks_on_top;
    pub mod prefer_jest_mocked;
    pub mod prefer_lowercase_title;
    pub mod prefer_mock_promise_shorthand;
    pub mod prefer_spy_on;
    pub mod prefer_strict_equal;
    pub mod prefer_to_be;
    pub mod prefer_to_contain;
    pub mod prefer_to_have_length;
    pub mod prefer_todo;
    pub mod require_hook;
    pub mod require_to_throw_message;
    pub mod require_top_level_describe;
    pub mod valid_describe_callback;
    pub mod valid_expect;
    pub mod valid_title;
}

mod react {
    pub mod button_has_type;
    pub mod checked_requires_onchange_or_readonly;
    pub mod exhaustive_deps;
    pub mod forbid_elements;
    pub mod forward_ref_uses_ref;
    pub mod iframe_missing_sandbox;
    pub mod jsx_boolean_value;
    pub mod jsx_curly_brace_presence;
    pub mod jsx_filename_extension;
    pub mod jsx_key;
    pub mod jsx_no_comment_textnodes;
    pub mod jsx_no_duplicate_props;
    pub mod jsx_no_script_url;
    pub mod jsx_no_target_blank;
    pub mod jsx_no_undef;
    pub mod jsx_no_useless_fragment;
    pub mod jsx_props_no_spread_multi;
    pub mod no_array_index_key;
    pub mod no_children_prop;
    pub mod no_danger;
    pub mod no_danger_with_children;
    pub mod no_direct_mutation_state;
    pub mod no_find_dom_node;
    pub mod no_is_mounted;
    pub mod no_namespace;
    pub mod no_render_return_value;
    pub mod no_set_state;
    pub mod no_string_refs;
    pub mod no_unescaped_entities;
    pub mod no_unknown_property;
    pub mod prefer_es6_class;
    pub mod react_in_jsx_scope;
    pub mod require_render_return;
    pub mod rules_of_hooks;
    pub mod self_closing_comp;
    pub mod style_prop_object;
    pub mod void_dom_elements_no_children;
}

mod react_perf {
    pub mod jsx_no_jsx_as_prop;
    pub mod jsx_no_new_array_as_prop;
    pub mod jsx_no_new_function_as_prop;
    pub mod jsx_no_new_object_as_prop;
}

mod unicorn {
    pub mod catch_error_name;
    pub mod consistent_assert;
    pub mod consistent_date_clone;
    pub mod consistent_empty_array_spread;
    pub mod consistent_existence_index_check;
    pub mod consistent_function_scoping;
    pub mod empty_brace_spaces;
    pub mod error_message;
    pub mod escape_case;
    pub mod explicit_length_check;
    pub mod filename_case;
    pub mod new_for_builtins;
    pub mod no_abusive_eslint_disable;
    pub mod no_accessor_recursion;
    pub mod no_anonymous_default_export;
    pub mod no_array_for_each;
    pub mod no_array_method_this_argument;
    pub mod no_array_reduce;
    pub mod no_await_expression_member;
    pub mod no_await_in_promise_methods;
    pub mod no_console_spaces;
    pub mod no_document_cookie;
    pub mod no_empty_file;
    pub mod no_hex_escape;
    pub mod no_instanceof_array;
    pub mod no_instanceof_builtins;
    pub mod no_invalid_fetch_options;
    pub mod no_invalid_remove_event_listener;
    pub mod no_length_as_slice_end;
    pub mod no_lonely_if;
    pub mod no_magic_array_flat_depth;
    pub mod no_negation_in_equality_check;
    pub mod no_nested_ternary;
    pub mod no_new_array;
    pub mod no_new_buffer;
    pub mod no_null;
    pub mod no_object_as_default_parameter;
    pub mod no_process_exit;
    pub mod no_single_promise_in_promise_methods;
    pub mod no_static_only_class;
    pub mod no_thenable;
    pub mod no_this_assignment;
    pub mod no_typeof_undefined;
    pub mod no_unnecessary_array_flat_depth;
    pub mod no_unnecessary_await;
    pub mod no_unnecessary_slice_end;
    pub mod no_unreadable_array_destructuring;
    pub mod no_unreadable_iife;
    pub mod no_useless_fallback_in_spread;
    pub mod no_useless_length_check;
    pub mod no_useless_promise_resolve_reject;
    pub mod no_useless_spread;
    pub mod no_useless_switch_case;
    pub mod no_useless_undefined;
    pub mod no_zero_fractions;
    pub mod number_literal_case;
    pub mod numeric_separators_style;
    pub mod prefer_add_event_listener;
    pub mod prefer_array_find;
    pub mod prefer_array_flat;
    pub mod prefer_array_flat_map;
    pub mod prefer_array_index_of;
    pub mod prefer_array_some;
    pub mod prefer_blob_reading_methods;
    pub mod prefer_code_point;
    pub mod prefer_date_now;
    pub mod prefer_dom_node_append;
    pub mod prefer_dom_node_dataset;
    pub mod prefer_dom_node_remove;
    pub mod prefer_dom_node_text_content;
    pub mod prefer_event_target;
    pub mod prefer_global_this;
    pub mod prefer_includes;
    pub mod prefer_logical_operator_over_ternary;
    pub mod prefer_math_min_max;
    pub mod prefer_math_trunc;
    pub mod prefer_modern_dom_apis;
    pub mod prefer_modern_math_apis;
    pub mod prefer_native_coercion_functions;
    pub mod prefer_negative_index;
    pub mod prefer_node_protocol;
    pub mod prefer_number_properties;
    pub mod prefer_object_from_entries;
    pub mod prefer_optional_catch_binding;
    pub mod prefer_prototype_methods;
    pub mod prefer_query_selector;
    pub mod prefer_reflect_apply;
    pub mod prefer_regexp_test;
    pub mod prefer_set_has;
    pub mod prefer_set_size;
    pub mod prefer_spread;
    pub mod prefer_string_raw;
    pub mod prefer_string_replace_all;
    pub mod prefer_string_slice;
    pub mod prefer_string_starts_ends_with;
    pub mod prefer_string_trim_start_end;
    pub mod prefer_structured_clone;
    pub mod prefer_type_error;
    pub mod require_array_join_separator;
    pub mod require_number_to_fixed_digits_argument;
    pub mod require_post_message_target_origin;
    pub mod switch_case_braces;
    pub mod text_encoding_identifier_case;
    pub mod throw_new_error;
}

mod jsx_a11y {
    pub mod alt_text;
    pub mod anchor_ambiguous_text;
    pub mod anchor_has_content;
    pub mod anchor_is_valid;
    pub mod aria_activedescendant_has_tabindex;
    pub mod aria_props;
    pub mod aria_role;
    pub mod aria_unsupported_elements;
    pub mod autocomplete_valid;
    pub mod click_events_have_key_events;
    pub mod heading_has_content;
    pub mod html_has_lang;
    pub mod iframe_has_title;
    pub mod img_redundant_alt;
    pub mod label_has_associated_control;
    pub mod lang;
    pub mod media_has_caption;
    pub mod mouse_events_have_key_events;
    pub mod no_access_key;
    pub mod no_aria_hidden_on_focusable;
    pub mod no_autofocus;
    pub mod no_distracting_elements;
    pub mod no_noninteractive_tabindex;
    pub mod no_redundant_roles;
    pub mod prefer_tag_over_role;
    pub mod role_has_required_aria_props;
    pub mod role_supports_aria_props;
    pub mod scope;
    pub mod tabindex_no_positive;
}

mod oxc {
    pub mod approx_constant;
    pub mod bad_array_method_on_arguments;
    pub mod bad_bitwise_operator;
    pub mod bad_char_at_comparison;
    pub mod bad_comparison_sequence;
    pub mod bad_min_max_func;
    pub mod bad_object_literal_comparison;
    pub mod bad_replace_all_arg;
    pub mod const_comparisons;
    pub mod double_comparisons;
    pub mod erasing_op;
    pub mod misrefactored_assign_op;
    pub mod missing_throw;
    pub mod no_accumulating_spread;
    pub mod no_async_await;
    pub mod no_async_endpoint_handlers;
    pub mod no_barrel_file;
    pub mod no_const_enum;
    pub mod no_map_spread;
    pub mod no_optional_chaining;
    pub mod no_rest_spread_properties;
    pub mod number_arg_out_of_range;
    pub mod only_used_in_recursion;
    pub mod uninvoked_array_callback;
}

mod nextjs {
    pub mod google_font_display;
    pub mod google_font_preconnect;
    pub mod inline_script_id;
    pub mod next_script_for_ga;
    pub mod no_assign_module_variable;
    pub mod no_async_client_component;
    pub mod no_before_interactive_script_outside_document;
    pub mod no_css_tags;
    pub mod no_document_import_in_page;
    pub mod no_duplicate_head;
    pub mod no_head_element;
    pub mod no_head_import_in_document;
    pub mod no_html_link_for_pages;
    pub mod no_img_element;
    pub mod no_page_custom_font;
    pub mod no_script_component_in_head;
    pub mod no_styled_jsx_in_document;
    pub mod no_sync_scripts;
    pub mod no_title_in_document_head;
    pub mod no_typos;
    pub mod no_unwanted_polyfillio;
}

/// <https://github.com/gajus/eslint-plugin-jsdoc>
mod jsdoc {
    pub mod check_access;
    pub mod check_property_names;
    pub mod check_tag_names;
    pub mod empty_tags;
    pub mod implements_on_classes;
    pub mod no_defaults;
    pub mod require_param;
    pub mod require_param_description;
    pub mod require_param_name;
    pub mod require_param_type;
    pub mod require_property;
    pub mod require_property_description;
    pub mod require_property_name;
    pub mod require_property_type;
    pub mod require_returns;
    pub mod require_returns_description;
    pub mod require_returns_type;
    pub mod require_yields;
}

mod promise {
    pub mod avoid_new;
    pub mod catch_or_return;
    pub mod no_callback_in_promise;
    pub mod no_nesting;
    pub mod no_new_statics;
    pub mod no_promise_in_callback;
    pub mod no_return_in_finally;
    pub mod no_return_wrap;
    pub mod param_names;
    pub mod prefer_await_to_callbacks;
    pub mod prefer_await_to_then;
    pub mod prefer_catch;
    pub mod spec_only;
    pub mod valid_params;
}

mod vitest {
    pub mod no_conditional_tests;
    pub mod no_import_node_test;
    pub mod prefer_to_be_falsy;
    pub mod prefer_to_be_object;
    pub mod prefer_to_be_truthy;
    pub mod require_local_test_context_for_concurrent_snapshots;
}

mod node {
    pub mod no_exports_assign;
    pub mod no_new_require;
}

oxc_macros::declare_all_lint_rules! {
    eslint::array_callback_return,
    eslint::arrow_body_style,
    eslint::block_scoped_var,
    eslint::curly,
    eslint::default_case,
    eslint::default_case_last,
    eslint::default_param_last,
    eslint::eqeqeq,
    eslint::for_direction,
    eslint::func_style,
    eslint::func_names,
    eslint::getter_return,
    eslint::grouped_accessor_pairs,
    eslint::guard_for_in,
    eslint::id_length,
    eslint::init_declarations,
    eslint::max_classes_per_file,
    eslint::max_depth,
    eslint::max_lines_per_function,
    eslint::max_lines,
    eslint::max_nested_callbacks,
    eslint::max_params,
    eslint::new_cap,
    eslint::no_extra_bind,
    eslint::no_alert,
    eslint::no_array_constructor,
    eslint::no_async_promise_executor,
    eslint::no_await_in_loop,
    eslint::no_bitwise,
    eslint::no_caller,
    eslint::no_case_declarations,
    eslint::no_class_assign,
    eslint::no_duplicate_imports,
    eslint::no_extra_label,
    eslint::no_labels,
    eslint::no_lone_blocks,
    eslint::no_lonely_if,
    eslint::no_multi_assign,
    eslint::no_nested_ternary,
    eslint::no_object_constructor,
    eslint::no_restricted_imports,
    eslint::no_unneeded_ternary,
    eslint::no_useless_backreference,
    eslint::no_useless_call,
    eslint::no_compare_neg_zero,
    eslint::no_cond_assign,
    eslint::no_console,
    eslint::no_const_assign,
    eslint::no_constant_binary_expression,
    eslint::no_constant_condition,
    eslint::no_constructor_return,
    eslint::no_continue,
    eslint::no_control_regex,
    eslint::no_debugger,
    eslint::no_delete_var,
    eslint::no_div_regex,
    eslint::no_dupe_class_members,
    eslint::no_dupe_else_if,
    eslint::no_dupe_keys,
    eslint::no_duplicate_case,
    eslint::no_else_return,
    eslint::no_empty_character_class,
    eslint::no_empty_function,
    eslint::no_empty_pattern,
    eslint::no_empty_static_block,
    eslint::no_empty,
    eslint::no_eq_null,
    eslint::no_eval,
    eslint::no_ex_assign,
    eslint::no_extend_native,
    eslint::no_extra_boolean_cast,
    eslint::no_fallthrough,
    eslint::no_func_assign,
    eslint::no_global_assign,
    eslint::no_import_assign,
    eslint::no_inner_declarations,
    eslint::no_invalid_regexp,
    eslint::no_irregular_whitespace,
    eslint::no_iterator,
    eslint::no_label_var,
    eslint::no_loss_of_precision,
    eslint::no_magic_numbers,
    eslint::no_negated_condition,
    eslint::no_multi_str,
    eslint::no_new_func,
    eslint::no_new_native_nonconstructor,
    eslint::no_new_wrappers,
    eslint::no_new,
    eslint::no_nonoctal_decimal_escape,
    eslint::no_obj_calls,
    eslint::no_plusplus,
    eslint::no_proto,
    eslint::no_prototype_builtins,
    eslint::no_redeclare,
    eslint::no_regex_spaces,
    eslint::no_restricted_globals,
    eslint::no_return_assign,
    eslint::no_script_url,
    eslint::no_self_assign,
    eslint::no_self_compare,
    eslint::no_setter_return,
    eslint::no_shadow_restricted_names,
    eslint::no_sparse_arrays,
    eslint::no_template_curly_in_string,
    eslint::no_ternary,
    eslint::no_this_before_super,
    eslint::no_throw_literal,
    eslint::no_undef,
    eslint::no_undefined,
    eslint::no_unexpected_multiline,
    eslint::no_unreachable,
    eslint::no_unsafe_finally,
    eslint::no_unsafe_negation,
    eslint::no_unsafe_optional_chaining,
    eslint::no_unused_expressions,
    eslint::no_unused_labels,
    eslint::no_unused_private_class_members,
    eslint::no_unused_vars,
    eslint::no_useless_catch,
    eslint::no_useless_concat,
    eslint::no_useless_constructor,
    eslint::no_useless_escape,
    eslint::no_useless_rename,
    eslint::no_var,
    eslint::no_void,
    eslint::no_with,
    eslint::operator_assignment,
    eslint::prefer_promise_reject_errors,
    eslint::prefer_exponentiation_operator,
    eslint::prefer_numeric_literals,
    eslint::prefer_object_has_own,
    eslint::prefer_object_spread,
    eslint::prefer_rest_params,
    eslint::prefer_spread,
    eslint::radix,
    eslint::require_await,
    eslint::require_yield,
    eslint::sort_imports,
    eslint::sort_keys,
    eslint::sort_vars,
    eslint::symbol_description,
    eslint::unicode_bom,
    eslint::use_isnan,
    eslint::valid_typeof,
    eslint::vars_on_top,
    eslint::yoda,
    import::consistent_type_specifier_style,
    eslint::complexity,
    eslint::no_warning_comments,
    import::default,
    import::export,
    import::exports_last,
    import::extensions,
    import::first,
    import::group_exports,
    import::no_unassigned_import,
    import::no_empty_named_blocks,
    import::no_anonymous_default_export,
    import::no_absolute_path,
    import::no_mutable_exports,
    import::no_named_default,
    import::no_namespace,
    import::max_dependencies,
    import::named,
    import::namespace,
    import::no_amd,
    import::no_commonjs,
    import::no_cycle,
    import::no_default_export,
    import::no_duplicates,
    import::no_dynamic_require,
    import::no_named_as_default,
    import::no_named_as_default_member,
    import::no_self_import,
    import::no_webpack_loader_syntax,
    import::prefer_default_export,
    import::unambiguous,
    jest::consistent_test_it,
    jest::expect_expect,
    jest::max_expects,
    jest::max_nested_describe,
    jest::no_alias_methods,
    jest::no_commented_out_tests,
    jest::no_conditional_expect,
    jest::no_conditional_in_test,
    jest::no_confusing_set_timeout,
    jest::no_deprecated_functions,
    jest::no_disabled_tests,
    jest::no_done_callback,
    jest::no_duplicate_hooks,
    jest::no_export,
    jest::no_focused_tests,
    jest::no_hooks,
    jest::no_identical_title,
    jest::no_interpolation_in_snapshots,
    jest::no_jasmine_globals,
    jest::no_large_snapshots,
    jest::no_mocks_import,
    jest::no_restricted_jest_methods,
    jest::no_restricted_matchers,
    jest::no_standalone_expect,
    jest::no_test_prefixes,
    jest::no_test_return_statement,
    jest::no_untyped_mock_factory,
    jest::prefer_each,
    jest::prefer_called_with,
    jest::prefer_comparison_matcher,
    jest::prefer_equality_matcher,
    jest::prefer_expect_resolves,
    jest::prefer_hooks_in_order,
    jest::prefer_hooks_on_top,
    jest::prefer_jest_mocked,
    jest::prefer_lowercase_title,
    jest::prefer_mock_promise_shorthand,
    jest::prefer_spy_on,
    jest::prefer_strict_equal,
    jest::prefer_to_be,
    jest::prefer_to_contain,
    jest::prefer_to_have_length,
    jest::prefer_todo,
    jest::require_hook,
    jest::require_to_throw_message,
    jest::require_top_level_describe,
    jest::valid_describe_callback,
    jest::valid_expect,
    jest::valid_title,
    jsdoc::check_access,
    jsdoc::check_property_names,
    jsdoc::check_tag_names,
    jsdoc::empty_tags,
    jsdoc::implements_on_classes,
    jsdoc::no_defaults,
    jsdoc::require_param,
    jsdoc::require_param_description,
    jsdoc::require_param_name,
    jsdoc::require_param_type,
    jsdoc::require_property,
    jsdoc::require_property_description,
    jsdoc::require_property_name,
    jsdoc::require_property_type,
    jsdoc::require_returns,
    jsdoc::require_returns_description,
    jsdoc::require_returns_type,
    jsdoc::require_yields,
    jsx_a11y::alt_text,
    jsx_a11y::anchor_has_content,
    jsx_a11y::anchor_is_valid,
    jsx_a11y::aria_activedescendant_has_tabindex,
    jsx_a11y::aria_props,
    jsx_a11y::aria_role,
    jsx_a11y::aria_unsupported_elements,
    jsx_a11y::autocomplete_valid,
    jsx_a11y::click_events_have_key_events,
    jsx_a11y::heading_has_content,
    jsx_a11y::html_has_lang,
    jsx_a11y::iframe_has_title,
    jsx_a11y::img_redundant_alt,
    jsx_a11y::label_has_associated_control,
    jsx_a11y::lang,
    jsx_a11y::media_has_caption,
    jsx_a11y::mouse_events_have_key_events,
    jsx_a11y::no_noninteractive_tabindex,
    jsx_a11y::no_access_key,
    jsx_a11y::no_aria_hidden_on_focusable,
    jsx_a11y::no_autofocus,
    jsx_a11y::no_distracting_elements,
    jsx_a11y::no_redundant_roles,
    jsx_a11y::prefer_tag_over_role,
    jsx_a11y::role_has_required_aria_props,
    jsx_a11y::role_supports_aria_props,
    jsx_a11y::scope,
    jsx_a11y::tabindex_no_positive,
    jsx_a11y::anchor_ambiguous_text,
    nextjs::google_font_display,
    nextjs::google_font_preconnect,
    nextjs::inline_script_id,
    nextjs::next_script_for_ga,
    nextjs::no_assign_module_variable,
    nextjs::no_async_client_component,
    nextjs::no_before_interactive_script_outside_document,
    nextjs::no_css_tags,
    nextjs::no_document_import_in_page,
    nextjs::no_duplicate_head,
    nextjs::no_head_element,
    nextjs::no_head_import_in_document,
    nextjs::no_img_element,
    nextjs::no_page_custom_font,
    nextjs::no_script_component_in_head,
    nextjs::no_styled_jsx_in_document,
    nextjs::no_sync_scripts,
    nextjs::no_title_in_document_head,
    nextjs::no_typos,
    nextjs::no_unwanted_polyfillio,
    nextjs::no_html_link_for_pages,
    node::no_exports_assign,
    node::no_new_require,
    oxc::approx_constant,
    oxc::bad_array_method_on_arguments,
    oxc::bad_bitwise_operator,
    oxc::bad_char_at_comparison,
    oxc::bad_comparison_sequence,
    oxc::bad_min_max_func,
    oxc::bad_object_literal_comparison,
    oxc::bad_replace_all_arg,
    oxc::const_comparisons,
    oxc::double_comparisons,
    oxc::erasing_op,
    oxc::misrefactored_assign_op,
    oxc::missing_throw,
    oxc::no_accumulating_spread,
    oxc::no_async_await,
    oxc::no_async_endpoint_handlers,
    oxc::no_barrel_file,
    oxc::no_const_enum,
    oxc::no_map_spread,
    oxc::no_optional_chaining,
    oxc::no_rest_spread_properties,
    oxc::number_arg_out_of_range,
    oxc::only_used_in_recursion,
    oxc::uninvoked_array_callback,
    promise::avoid_new,
    promise::catch_or_return,
    promise::no_return_wrap,
    promise::no_nesting,
    promise::no_promise_in_callback,
    promise::no_callback_in_promise,
    promise::no_new_statics,
    promise::no_return_in_finally,
    promise::param_names,
    promise::prefer_catch,
    promise::prefer_await_to_callbacks,
    promise::prefer_await_to_then,
    promise::spec_only,
    promise::valid_params,
    react::button_has_type,
    react::checked_requires_onchange_or_readonly,
    react::exhaustive_deps,
    react::forbid_elements,
    react::forward_ref_uses_ref,
    react::iframe_missing_sandbox,
    react::jsx_filename_extension,
    react::jsx_boolean_value,
    react::jsx_curly_brace_presence,
    react::jsx_key,
    react::jsx_no_comment_textnodes,
    react::jsx_no_duplicate_props,
    react::jsx_no_script_url,
    react::jsx_no_target_blank,
    react::jsx_no_undef,
    react::jsx_no_useless_fragment,
    react::jsx_props_no_spread_multi,
    react::no_namespace,
    react::no_array_index_key,
    react::no_children_prop,
    react::no_danger_with_children,
    react::no_danger,
    react::no_direct_mutation_state,
    react::no_find_dom_node,
    react::no_is_mounted,
    react::no_render_return_value,
    react::no_set_state,
    react::no_string_refs,
    react::no_unescaped_entities,
    react::no_unknown_property,
    react::prefer_es6_class,
    react::react_in_jsx_scope,
    react::require_render_return,
    react::rules_of_hooks,
    react::self_closing_comp,
    react::style_prop_object,
    react::void_dom_elements_no_children,
    react_perf::jsx_no_jsx_as_prop,
    react_perf::jsx_no_new_array_as_prop,
    react_perf::jsx_no_new_function_as_prop,
    react_perf::jsx_no_new_object_as_prop,
    typescript::adjacent_overload_signatures,
    typescript::array_type,
    typescript::ban_ts_comment,
    typescript::ban_tslint_comment,
    typescript::ban_types,
    typescript::consistent_generic_constructors,
    typescript::consistent_indexed_object_style,
    typescript::consistent_type_definitions,
    typescript::consistent_type_imports,
    typescript::explicit_module_boundary_types,
    typescript::explicit_function_return_type,
    typescript::no_inferrable_types,
    typescript::no_confusing_non_null_assertion,
    typescript::no_duplicate_enum_values,
    typescript::no_dynamic_delete,
    typescript::no_empty_interface,
    typescript::no_empty_object_type,
    typescript::no_explicit_any,
    typescript::no_extra_non_null_assertion,
    typescript::no_extraneous_class,
    typescript::no_import_type_side_effects,
    typescript::no_misused_new,
    typescript::no_namespace,
    typescript::no_non_null_asserted_nullish_coalescing,
    typescript::no_non_null_asserted_optional_chain,
    typescript::no_non_null_assertion,
    typescript::no_require_imports,
    typescript::no_this_alias,
    typescript::no_unnecessary_parameter_property_assignment,
    typescript::no_unnecessary_type_constraint,
    typescript::no_unsafe_declaration_merging,
    typescript::no_unsafe_function_type,
    typescript::no_useless_empty_export,
    typescript::no_var_requires,
    typescript::no_wrapper_object_types,
    typescript::prefer_as_const,
    typescript::prefer_enum_initializers,
    typescript::prefer_for_of,
    typescript::prefer_function_type,
    typescript::prefer_literal_enum_member,
    typescript::prefer_namespace_keyword,
    typescript::prefer_ts_expect_error,
    typescript::triple_slash_reference,
    unicorn::catch_error_name,
    unicorn::consistent_assert,
    unicorn::consistent_date_clone,
    unicorn::consistent_empty_array_spread,
    unicorn::consistent_existence_index_check,
    unicorn::consistent_function_scoping,
    unicorn::empty_brace_spaces,
    unicorn::error_message,
    unicorn::escape_case,
    unicorn::explicit_length_check,
    unicorn::filename_case,
    unicorn::new_for_builtins,
    unicorn::no_instanceof_builtins,
    unicorn::no_array_method_this_argument,
    unicorn::no_unnecessary_array_flat_depth,
    unicorn::no_unnecessary_slice_end,
    unicorn::no_accessor_recursion,
    unicorn::no_invalid_fetch_options,
    unicorn::no_abusive_eslint_disable,
    unicorn::no_anonymous_default_export,
    unicorn::no_array_for_each,
    unicorn::no_array_reduce,
    unicorn::no_await_expression_member,
    unicorn::no_await_in_promise_methods,
    unicorn::no_console_spaces,
    unicorn::no_document_cookie,
    unicorn::no_empty_file,
    unicorn::no_hex_escape,
    unicorn::no_instanceof_array,
    unicorn::no_invalid_remove_event_listener,
    unicorn::no_length_as_slice_end,
    unicorn::no_lonely_if,
    unicorn::no_magic_array_flat_depth,
    unicorn::no_negation_in_equality_check,
    unicorn::no_nested_ternary,
    unicorn::no_new_array,
    unicorn::no_new_buffer,
    unicorn::no_null,
    unicorn::no_object_as_default_parameter,
    unicorn::no_process_exit,
    unicorn::no_single_promise_in_promise_methods,
    unicorn::no_static_only_class,
    unicorn::no_thenable,
    unicorn::no_this_assignment,
    unicorn::no_typeof_undefined,
    unicorn::no_unnecessary_await,
    unicorn::no_unreadable_array_destructuring,
    unicorn::no_unreadable_iife,
    unicorn::no_useless_fallback_in_spread,
    unicorn::no_useless_length_check,
    unicorn::no_useless_promise_resolve_reject,
    unicorn::no_useless_spread,
    unicorn::no_useless_switch_case,
    unicorn::no_useless_undefined,
    unicorn::no_zero_fractions,
    unicorn::number_literal_case,
    unicorn::numeric_separators_style,
    unicorn::prefer_global_this,
    unicorn::prefer_object_from_entries,
    unicorn::prefer_array_find,
    unicorn::prefer_array_index_of,
    unicorn::prefer_spread,
    unicorn::prefer_add_event_listener,
    unicorn::prefer_array_flat,
    unicorn::prefer_array_flat_map,
    unicorn::prefer_array_some,
    unicorn::prefer_blob_reading_methods,
    unicorn::prefer_code_point,
    unicorn::prefer_date_now,
    unicorn::prefer_dom_node_append,
    unicorn::prefer_dom_node_dataset,
    unicorn::prefer_dom_node_remove,
    unicorn::prefer_dom_node_text_content,
    unicorn::prefer_event_target,
    unicorn::prefer_includes,
    unicorn::prefer_logical_operator_over_ternary,
    unicorn::prefer_math_min_max,
    unicorn::prefer_math_trunc,
    unicorn::prefer_modern_dom_apis,
    unicorn::prefer_modern_math_apis,
    unicorn::prefer_native_coercion_functions,
    unicorn::prefer_negative_index,
    unicorn::prefer_node_protocol,
    unicorn::prefer_number_properties,
    unicorn::prefer_optional_catch_binding,
    unicorn::prefer_prototype_methods,
    unicorn::prefer_query_selector,
    unicorn::prefer_reflect_apply,
    unicorn::prefer_regexp_test,
    unicorn::prefer_set_has,
    unicorn::prefer_set_size,
    unicorn::prefer_string_raw,
    unicorn::prefer_string_replace_all,
    unicorn::prefer_string_slice,
    unicorn::prefer_string_starts_ends_with,
    unicorn::prefer_string_trim_start_end,
    unicorn::prefer_structured_clone,
    unicorn::prefer_type_error,
    unicorn::require_post_message_target_origin,
    unicorn::require_array_join_separator,
    unicorn::require_number_to_fixed_digits_argument,
    unicorn::switch_case_braces,
    unicorn::text_encoding_identifier_case,
    unicorn::throw_new_error,
    vitest::no_conditional_tests,
    vitest::no_import_node_test,
    vitest::prefer_to_be_falsy,
    vitest::prefer_to_be_object,
    vitest::prefer_to_be_truthy,
    vitest::require_local_test_context_for_concurrent_snapshots,
}
