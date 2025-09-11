use crate::ast::context::context_object::ContextObject;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::functions::function_date::*;
use crate::ast::functions::function_numeric::*;
use crate::ast::functions::function_string::*;
use crate::ast::functions::function_list::*;
use crate::ast::token::ExpressionEnum;
use crate::ast::utils::array_to_code_sep;
use crate::ast::{is_linked, Link};
use crate::runtime::execution_context::*;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::{ValueType, TypedValue};
use crate::typesystem::values::ValueEnum;
use log::error;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use phf::phf_map;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionHolder<T, V, R> {
    pub name: &'static str,
    pub function: T,
    pub validation: V,
    pub return_type: R,
}

pub type UnaryFunctionDefinition = FunctionHolder<
    fn(ValueEnum) -> Result<ValueEnum, RuntimeError>,
    fn(ValueType) -> Link<()>,
    fn(ValueType) -> ValueType,
>;

pub type BinaryFunctionDefinition = FunctionHolder<
    fn(ValueEnum, ValueEnum) -> Result<ValueEnum, RuntimeError>,
    fn(ValueType, ValueType) -> Link<()>,
    fn(ValueType, ValueType) -> ValueType,
>;

/// validation method will receive all item types for all arguments
pub type MultiFunctionDefinition = FunctionHolder<
    fn(Vec<Result<ValueEnum, RuntimeError>>, ValueType) -> Result<ValueEnum, RuntimeError>,
    fn(Vec<ValueType>) -> Link<()>,
    fn() -> ValueType,
>;

pub static UNARY_BUILT_IN_FUNCTIONS: phf::Map<&'static str, UnaryFunctionDefinition> = phf_map! {
    "count" => UnaryFunctionDefinition { name : "count", function: eval_count, validation: number_range_or_number_list, return_type: return_uni_number },
    "max" => UnaryFunctionDefinition { name : "max", function: eval_max, validation: number_range_or_number_list, return_type: return_uni_number },
    "sum" => UnaryFunctionDefinition { name : "sum", function: eval_sum, validation: number_range_or_number_list, return_type: return_uni_number },
    // List numerics
    "min" => UnaryFunctionDefinition { name : "min", function: eval_min, validation: validate_unary_list_numbers, return_type: return_uni_number },
    "product" => UnaryFunctionDefinition { name : "product", function: eval_product, validation: validate_unary_list_numbers, return_type: return_uni_number },
    "mean" => UnaryFunctionDefinition { name : "mean", function: eval_mean, validation: validate_unary_list_numbers, return_type: return_uni_number },
    "median" => UnaryFunctionDefinition { name : "median", function: eval_median, validation: validate_unary_list_numbers, return_type: return_uni_number },
    "stddev" => UnaryFunctionDefinition { name : "stddev", function: eval_stddev, validation: validate_unary_list_numbers, return_type: return_uni_number },
    "mode" => UnaryFunctionDefinition { name : "mode", function: eval_mode, validation: validate_unary_list, return_type: |_| ValueType::ListType(Box::new(ValueType::NumberType)) },
    // Booleans
    "all" => UnaryFunctionDefinition { name : "all", function: eval_all, validation: |v| { if let ValueType::ListType(inner) = v { LinkingError::expect_type(None, *inner, &[ValueType::BooleanType]).map(|_| ()) } else { LinkingError::expect_type(None, v, &[ValueType::ListType(Box::new(ValueType::BooleanType))]).map(|_| ()) } }, return_type: |_| ValueType::BooleanType },
    "any" => UnaryFunctionDefinition { name : "any", function: eval_any, validation: |v| { if let ValueType::ListType(inner) = v { LinkingError::expect_type(None, *inner, &[ValueType::BooleanType]).map(|_| ()) } else { LinkingError::expect_type(None, v, &[ValueType::ListType(Box::new(ValueType::BooleanType))]).map(|_| ()) } }, return_type: |_| ValueType::BooleanType },
    // Date/Time/Duration parsing
    "date" => UnaryFunctionDefinition { name : "date", function: eval_date, validation: expect_string_arg, return_type: |_| ValueType::DateType },
    "time" => UnaryFunctionDefinition { name : "time", function: eval_time, validation: expect_string_arg, return_type: |_| ValueType::TimeType },
    "datetime" => UnaryFunctionDefinition { name : "datetime", function: eval_datetime, validation: expect_string_arg, return_type: |_| ValueType::DateTimeType },
    "duration" => UnaryFunctionDefinition { name : "duration", function: eval_duration, validation: expect_string_arg, return_type: |_| ValueType::DurationType },
    // Additional helpers
    "dayOfWeek" => UnaryFunctionDefinition { name : "dayOfWeek", function: eval_day_of_week, validation: expect_date_arg, return_type: |_| ValueType::StringType },
    "monthOfYear" => UnaryFunctionDefinition { name : "monthOfYear", function: eval_month_of_year, validation: expect_date_arg, return_type: |_| ValueType::StringType },
    "lastDayOfMonth" => UnaryFunctionDefinition { name : "lastDayOfMonth", function: eval_last_day_of_month, validation: expect_date_arg, return_type: |_| ValueType::NumberType },
    // String unary
    "length" => UnaryFunctionDefinition { name: "length", function: eval_length, validation: validate_unary_string, return_type: return_uni_number },
    "toUpperCase" => UnaryFunctionDefinition { name: "toUpperCase", function: eval_to_upper, validation: validate_unary_string, return_type: return_string_type_unary },
    "toLowerCase" => UnaryFunctionDefinition { name: "toLowerCase", function: eval_to_lower, validation: validate_unary_string, return_type: return_string_type_unary },
    "trim" => UnaryFunctionDefinition { name: "trim", function: eval_trim, validation: validate_unary_string, return_type: return_string_type_unary },
    // base64 group (optional)
    #[cfg(feature = "base64_functions")]
    "toBase64" => UnaryFunctionDefinition { name: "toBase64", function: eval_to_base64, validation: validate_unary_string, return_type: return_string_type_unary },
    #[cfg(feature = "base64_functions")]
    "fromBase64" => UnaryFunctionDefinition { name: "fromBase64", function: eval_from_base64, validation: validate_unary_string, return_type: return_string_type_unary },
    // reverse for string or list
    "reverse" => UnaryFunctionDefinition { name: "reverse", function: eval_reverse_mixed, validation: validate_unary_reverse_mixed, return_type: return_same_list_type },
    "sanitizeFilename" => UnaryFunctionDefinition { name: "sanitizeFilename", function: eval_sanitize_filename, validation: validate_unary_string, return_type: return_string_type_unary },
    // list helpers
    "distinctValues" => UnaryFunctionDefinition { name: "distinctValues", function: eval_distinct, validation: validate_unary_list, return_type: return_same_list_type },
    "duplicateValues" => UnaryFunctionDefinition { name: "duplicateValues", function: eval_duplicates, validation: validate_unary_list, return_type: return_same_list_type },
    "flatten" => UnaryFunctionDefinition { name: "flatten", function: eval_flatten, validation: validate_unary_list, return_type: return_flatten_type },
    "isEmpty" => UnaryFunctionDefinition { name: "isEmpty", function: eval_is_empty, validation: validate_unary_list, return_type: |_| ValueType::BooleanType },
};

pub static BINARY_BUILT_IN_FUNCTIONS: phf::Map<&'static str, BinaryFunctionDefinition> = phf_map! {
    "find" => BinaryFunctionDefinition { name : "find", function: eval_find, validation: list_item_as_second_arg, return_type: return_binary_same_as_right_arg },
    // List or String
    "contains" => BinaryFunctionDefinition { name: "contains", function: eval_contains_mixed, validation: validate_binary_contains_mixed, return_type: return_boolean_type_binary },
    "startsWith" => BinaryFunctionDefinition { name: "startsWith", function: eval_starts_with, validation: validate_binary_string_string, return_type: return_boolean_type_binary },
    "endsWith" => BinaryFunctionDefinition { name: "endsWith", function: eval_ends_with, validation: validate_binary_string_string, return_type: return_boolean_type_binary },
    // split: regex when enabled, otherwise simple substring split
    "split" => BinaryFunctionDefinition { name: "split", function: eval_split, validation: validate_binary_string_string, return_type: return_string_list_type_binary },
    #[cfg(feature = "regex_functions")]
    "regexSplit" => BinaryFunctionDefinition { name: "regexSplit", function: eval_regex_split, validation: validate_binary_string_string, return_type: return_string_list_type_binary },
    "substringBefore" => BinaryFunctionDefinition { name: "substringBefore", function: eval_substring_before, validation: validate_binary_string_string, return_type: return_string_type_binary },
    "substringAfter" => BinaryFunctionDefinition { name: "substringAfter", function: eval_substring_after, validation: validate_binary_string_string, return_type: return_string_type_binary },
    "charAt" => BinaryFunctionDefinition { name: "charAt", function: eval_char_at, validation: validate_binary_string_number, return_type: return_string_type_binary },
    "charCodeAt" => BinaryFunctionDefinition { name: "charCodeAt", function: eval_char_code_at, validation: validate_binary_string_number, return_type: return_number_type_binary },
    // Mix: string or list
    "indexOf" => BinaryFunctionDefinition { name: "indexOf", function: eval_index_of_mixed, validation: validate_binary_index_of_mixed, return_type: return_index_of_type },
    "lastIndexOf" => BinaryFunctionDefinition { name: "lastIndexOf", function: eval_last_index_of, validation: validate_binary_string_string, return_type: return_number_type_binary },
    "repeat" => BinaryFunctionDefinition { name: "repeat", function: eval_repeat, validation: validate_binary_string_number, return_type: return_string_type_binary },
    "interpolate" => BinaryFunctionDefinition { name: "interpolate", function: eval_interpolate, validation: validate_binary_string_any, return_type: return_string_type_binary },
    // List-specific
    "remove" => BinaryFunctionDefinition { name: "remove", function: eval_remove, validation: validate_binary_list_number, return_type: |l, _| l },
    "sort" => BinaryFunctionDefinition { name: "sort", function: eval_sort, validation: validate_binary_sort, return_type: |l, _| l },
    "partition" => BinaryFunctionDefinition { name: "partition", function: eval_partition, validation: validate_binary_partition, return_type: |l, r| super::function_list::return_partition_type(l, r) },
};

pub static MULTI_BUILT_IN_FUNCTIONS: phf::Map<&'static str, MultiFunctionDefinition> = phf_map! {
    "max" => MultiFunctionDefinition { name : "max", function: eval_max_all, validation: validate_multi_all_args_numbers, return_type: return_multi_number },
    "sum" => MultiFunctionDefinition { name : "sum", function: eval_sum_all, validation: validate_multi_all_args_numbers, return_type: return_multi_number },
    "min" => MultiFunctionDefinition { name : "min", function: |vals, _| {
        let mut best: Option<crate::typesystem::types::number::NumberEnum> = None;
        for r in vals { let val = r?; if let ValueEnum::NumberValue(n) = val { best = Some(match best { Some(b) if n < b => n, Some(b) => b, None => n }); } else { return RuntimeError::type_not_supported(val.get_type()).into(); } }
        Ok(ValueEnum::NumberValue(best.unwrap_or(crate::typesystem::types::number::NumberEnum::from(0))))
    }, validation: validate_multi_all_args_numbers, return_type: return_multi_number },
    // List multi-arity
    "sublist" => MultiFunctionDefinition { name: "sublist", function: eval_sublist, validation: validate_multi_sublist, return_type: return_list_undefined },
    "append" => MultiFunctionDefinition { name: "append", function: eval_append, validation: validate_multi_append, return_type: return_list_undefined },
    "concatenate" => MultiFunctionDefinition { name: "concatenate", function: eval_concatenate, validation: validate_multi_concatenate, return_type: return_list_undefined },
    "insertBefore" => MultiFunctionDefinition { name: "insertBefore", function: eval_insert_before, validation: validate_multi_insert_before, return_type: return_list_undefined },
    "union" => MultiFunctionDefinition { name: "union", function: eval_union, validation: validate_multi_union, return_type: return_list_undefined },
    // String multi-arity
    "join" => MultiFunctionDefinition { name: "join", function: eval_join, validation: validate_multi_join, return_type: return_string_type_multi },
    "substring" => MultiFunctionDefinition { name: "substring", function: eval_substring, validation: validate_multi_substring, return_type: return_string_type_multi },
    "replace" => MultiFunctionDefinition { name: "replace", function: eval_replace, validation: validate_multi_replace, return_type: return_string_type_multi },
    #[cfg(feature = "regex_functions")]
    "regexReplace" => MultiFunctionDefinition { name: "regexReplace", function: eval_regex_replace, validation: validate_multi_replace, return_type: return_string_type_multi },
    "replaceFirst" => MultiFunctionDefinition { name: "replaceFirst", function: eval_replace_first, validation: validate_multi_replace, return_type: return_string_type_multi },
    "replaceLast" => MultiFunctionDefinition { name: "replaceLast", function: eval_replace_last, validation: validate_multi_replace, return_type: return_string_type_multi },
    "fromCharCode" => MultiFunctionDefinition { name: "fromCharCode", function: eval_from_char_code, validation: validate_multi_from_char_code, return_type: return_string_type_multi },
    "padStart" => MultiFunctionDefinition { name: "padStart", function: eval_pad_start, validation: validate_multi_pad, return_type: return_string_type_multi },
    "padEnd" => MultiFunctionDefinition { name: "padEnd", function: eval_pad_end, validation: validate_multi_pad, return_type: return_string_type_multi },
};

#[derive(Debug, PartialEq, Clone)]
pub enum EFunctionType {
    Unary,
    Binary,
    Multi,
    Custom(u8),
}

pub static BUILT_IN_ALL_FUNCTIONS: phf::Map<&'static str, EFunctionType> = phf_map! {
    "max" => EFunctionType::Multi,
    "sum" => EFunctionType::Multi,
    "min" => EFunctionType::Multi,
    "count" => EFunctionType::Unary,
    "find" => EFunctionType::Binary,
    "product" => EFunctionType::Unary,
    "mean" => EFunctionType::Unary,
    "median" => EFunctionType::Unary,
    "stddev" => EFunctionType::Unary,
    "mode" => EFunctionType::Unary,
    "all" => EFunctionType::Unary,
    "any" => EFunctionType::Unary,
    "sublist" => EFunctionType::Multi,
    "append" => EFunctionType::Multi,
    "concatenate" => EFunctionType::Multi,
    "insertBefore" => EFunctionType::Multi,
    "remove" => EFunctionType::Binary,
    "reverse" => EFunctionType::Unary,
    "indexOf" => EFunctionType::Binary,
    "union" => EFunctionType::Multi,
    "distinctValues" => EFunctionType::Unary,
    "duplicateValues" => EFunctionType::Unary,
    "flatten" => EFunctionType::Unary,
    "sort" => EFunctionType::Binary,
    "join" => EFunctionType::Multi,
    "isEmpty" => EFunctionType::Unary,
    "partition" => EFunctionType::Binary,
    // Date/Time/Duration parsing and helpers
    "date" => EFunctionType::Unary,
    "time" => EFunctionType::Unary,
    "datetime" => EFunctionType::Unary,
    "duration" => EFunctionType::Unary,
    "dayOfWeek" => EFunctionType::Unary,
    "monthOfYear" => EFunctionType::Unary,
    "lastDayOfMonth" => EFunctionType::Unary,
    // String
    "length" => EFunctionType::Unary,
    "toUpperCase" => EFunctionType::Unary,
    "toLowerCase" => EFunctionType::Unary,
    "trim" => EFunctionType::Unary,
    #[cfg(feature = "base64_functions")]
    "toBase64" => EFunctionType::Unary,
    #[cfg(feature = "base64_functions")]
    "fromBase64" => EFunctionType::Unary,
    // reverse accounted above
    "sanitizeFilename" => EFunctionType::Unary,
    // contains accounted above
    "startsWith" => EFunctionType::Binary,
    "endsWith" => EFunctionType::Binary,
    "split" => EFunctionType::Binary,
    #[cfg(feature = "regex_functions")]
    "regexSplit" => EFunctionType::Binary,
    "substringBefore" => EFunctionType::Binary,
    "substringAfter" => EFunctionType::Binary,
    "charAt" => EFunctionType::Binary,
    "charCodeAt" => EFunctionType::Binary,
    // indexOf accounted above
    "lastIndexOf" => EFunctionType::Binary,
    "repeat" => EFunctionType::Binary,
    "interpolate" => EFunctionType::Binary,
    "substring" => EFunctionType::Multi,
    "replace" => EFunctionType::Multi,
    #[cfg(feature = "regex_functions")]
    "regexReplace" => EFunctionType::Multi,
    // Basic variants always available
    "replaceFirst" => EFunctionType::Multi,
    "replaceLast" => EFunctionType::Multi,
    "fromCharCode" => EFunctionType::Multi,
    "padStart" => EFunctionType::Multi,
    "padEnd" => EFunctionType::Multi,
};

#[derive(Debug)]
pub struct BinaryFunction {
    pub left: ExpressionEnum,
    pub right: ExpressionEnum,
    pub definition: BinaryFunctionDefinition,
    pub return_type: Link<ValueType>,
}

impl BinaryFunction {
    pub fn build(
        definition: BinaryFunctionDefinition,
        left: ExpressionEnum,
        right: ExpressionEnum,
    ) -> Self {
        BinaryFunction {
            left,
            right,
            definition,
            return_type: LinkingError::not_linked().into(),
        }
    }
}

impl Display for BinaryFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}({},{})", self.definition.name, self.left, self.right)
    }
}

impl EvaluatableExpression for BinaryFunction {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        (self.definition.function)(self.left.eval(context.clone())?, self.right.eval(context)?)
    }
}

impl StaticLink for BinaryFunction {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        if !is_linked(&self.return_type) {
            error!("Linking find(...) function: {},{}", self.left, self.right);

            let left_type = self.left.link(Rc::clone(&ctx))?;
            let right_type = self.right.link(Rc::clone(&ctx))?;

            (self.definition.validation)(left_type.clone(), right_type.clone())?;

            self.return_type = Ok((self.definition.return_type)(left_type, right_type));
        }
        self.return_type.clone()
    }
}

//--------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct UnaryFunction {
    pub arg: ExpressionEnum,
    pub definition: UnaryFunctionDefinition,
    pub return_type: Link<ValueType>,
}

impl UnaryFunction {
    pub fn build(definition: UnaryFunctionDefinition, arg: ExpressionEnum) -> Self {
        UnaryFunction {
            arg,
            definition,
            return_type: LinkingError::not_linked().into(),
        }
    }
}

impl Display for UnaryFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.definition.name, self.arg)
    }
}

impl StaticLink for UnaryFunction {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        if !is_linked(&self.return_type) {
            let arg_type = self.arg.link(Rc::clone(&ctx))?;

            (self.definition.validation)(arg_type.clone())?;

            self.return_type = Ok((self.definition.return_type)(arg_type));
        }

        self.return_type.clone()
    }
}

impl EvaluatableExpression for UnaryFunction {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        (self.definition.function)(self.arg.eval(context)?)
    }
}

//--------------------------------------------------------------------------------------------------

/// **Multi function**
/// 1. Must have at least one argument
/// 2. All arguments must be of the same type
/// 3. Return type is the same as the argument type
#[derive(Debug)]
pub struct MultiFunction {
    pub args: Vec<ExpressionEnum>,
    pub definition: MultiFunctionDefinition,
    pub return_type: Link<ValueType>,
}

impl MultiFunction {
    pub fn build(definition: MultiFunctionDefinition, args: Vec<ExpressionEnum>) -> Self {
        MultiFunction {
            args,
            definition,
            return_type: LinkingError::not_linked().into(),
        }
    }
}

impl Display for MultiFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.definition.name,
            array_to_code_sep(self.args.iter(), ", ")
        )
    }
}

impl StaticLink for MultiFunction {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        if !is_linked(&self.return_type) {
            let mut arg_types = Vec::new();
            for arg in self.args.iter_mut() {
                arg_types.push(arg.link(Rc::clone(&ctx))?);
            }

            (self.definition.validation)(arg_types)?;

            self.return_type = Ok((self.definition.return_type)());
        }

        self.return_type.clone()
    }
}

impl EvaluatableExpression for MultiFunction {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let mut values = Vec::new();
        for arg in self.args.iter() {
            values.push(arg.eval(Rc::clone(&context)));
        }

        (self.definition.function)(values, self.return_type.clone()?)
    }
}
