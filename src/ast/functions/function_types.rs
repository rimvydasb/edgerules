use crate::ast::context::context_object::ContextObject;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::functions::function_date::*;
use crate::ast::functions::function_list::*;
use crate::ast::functions::function_numeric::*;
use crate::ast::functions::function_string::*;
use crate::ast::token::ExpressionEnum;
use crate::ast::utils::array_to_code_sep;
use crate::ast::{is_linked, Link};
use crate::runtime::execution_context::*;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::ValueType;
use crate::typesystem::values::ValueEnum;
use log::error;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(Debug)]
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

#[inline]
fn validate_unary_any(_: ValueType) -> Link<()> {
    Ok(())
}

#[inline]
fn return_uni_string(_: ValueType) -> ValueType {
    ValueType::StringType
}

#[inline]
fn return_uni_boolean(_: ValueType) -> ValueType {
    ValueType::BooleanType
}

#[inline]
fn return_uni_date(_: ValueType) -> ValueType {
    ValueType::DateType
}

#[inline]
fn return_uni_time(_: ValueType) -> ValueType {
    ValueType::TimeType
}

#[inline]
fn return_uni_datetime(_: ValueType) -> ValueType {
    ValueType::DateTimeType
}

#[inline]
fn return_uni_duration(_: ValueType) -> ValueType {
    ValueType::DurationType
}

#[inline]
fn return_uni_mode_number_list(_: ValueType) -> ValueType {
    ValueType::ListType(Some(Box::new(ValueType::NumberType)))
}

#[inline]
fn return_binary_same_as_left_arg(left: ValueType, _: ValueType) -> ValueType {
    left
}

pub const U_TO_STRING: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "toString",
    function: eval_to_string,
    validation: validate_unary_any,
    return_type: return_uni_string,
};

pub const U_COUNT: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "count",
    function: eval_count,
    validation: number_range_or_any_list,
    return_type: return_uni_number,
};

pub const U_MAX: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "max",
    function: eval_max,
    validation: number_range_or_number_list,
    return_type: return_uni_number,
};

pub const U_SUM: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "sum",
    function: eval_sum,
    validation: number_range_or_number_list,
    return_type: return_uni_number,
};

pub const U_MIN: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "min",
    function: eval_min,
    validation: number_range_or_number_list,
    return_type: return_uni_number,
};

pub const U_PRODUCT: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "product",
    function: eval_product,
    validation: validate_unary_list_numbers,
    return_type: return_uni_number,
};

pub const U_MEAN: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "mean",
    function: eval_mean,
    validation: validate_unary_list_numbers,
    return_type: return_uni_number,
};

pub const U_MEDIAN: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "median",
    function: eval_median,
    validation: validate_unary_list_numbers,
    return_type: return_uni_number,
};

pub const U_STDDEV: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "stddev",
    function: eval_stddev,
    validation: validate_unary_list_numbers,
    return_type: return_uni_number,
};

pub const U_MODE: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "mode",
    function: eval_mode,
    validation: validate_unary_list,
    return_type: return_uni_mode_number_list,
};

pub const U_DATE: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "date",
    function: eval_date,
    validation: expect_string_arg,
    return_type: return_uni_date,
};

pub const U_TIME: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "time",
    function: eval_time,
    validation: expect_string_arg,
    return_type: return_uni_time,
};

pub const U_DATETIME: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "datetime",
    function: eval_datetime,
    validation: expect_string_arg,
    return_type: return_uni_datetime,
};

pub const U_DURATION: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "duration",
    function: eval_duration,
    validation: expect_string_arg,
    return_type: return_uni_duration,
};

pub const U_DAY_OF_WEEK: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "dayOfWeek",
    function: eval_day_of_week,
    validation: expect_date_arg,
    return_type: return_uni_string,
};

pub const U_MONTH_OF_YEAR: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "monthOfYear",
    function: eval_month_of_year,
    validation: expect_date_arg,
    return_type: return_uni_string,
};

pub const U_LAST_DAY_OF_MONTH: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "lastDayOfMonth",
    function: eval_last_day_of_month,
    validation: expect_date_arg,
    return_type: return_uni_number,
};

pub const U_LENGTH: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "length",
    function: eval_length,
    validation: validate_unary_string,
    return_type: return_uni_number,
};

pub const U_TO_UPPER_CASE: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "toUpperCase",
    function: eval_to_upper,
    validation: validate_unary_string,
    return_type: return_string_type_unary,
};

pub const U_TO_LOWER_CASE: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "toLowerCase",
    function: eval_to_lower,
    validation: validate_unary_string,
    return_type: return_string_type_unary,
};

pub const U_TRIM: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "trim",
    function: eval_trim,
    validation: validate_unary_string,
    return_type: return_string_type_unary,
};

pub const U_TO_BASE64: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "toBase64",
    function: eval_to_base64,
    validation: validate_unary_string,
    return_type: return_string_type_unary,
};

pub const U_FROM_BASE64: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "fromBase64",
    function: eval_from_base64,
    validation: validate_unary_string,
    return_type: return_string_type_unary,
};

pub const U_REVERSE: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "reverse",
    function: eval_reverse_mixed,
    validation: validate_unary_reverse_mixed,
    return_type: return_same_list_type,
};

pub const U_SORT: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "sort",
    function: eval_sort,
    validation: validate_unary_list,
    return_type: return_same_list_type,
};

pub const U_SORT_DESCENDING: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "sortDescending",
    function: eval_sort_desc,
    validation: validate_unary_list,
    return_type: return_same_list_type,
};

pub const U_SANITIZE_FILENAME: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "sanitizeFilename",
    function: eval_sanitize_filename,
    validation: validate_unary_string,
    return_type: return_string_type_unary,
};

pub const U_DISTINCT_VALUES: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "distinctValues",
    function: eval_distinct,
    validation: validate_unary_list,
    return_type: return_same_list_type,
};

pub const U_DUPLICATE_VALUES: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "duplicateValues",
    function: eval_duplicates,
    validation: validate_unary_list,
    return_type: return_same_list_type,
};

pub const U_FLATTEN: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "flatten",
    function: eval_flatten,
    validation: validate_unary_list,
    return_type: return_flatten_type,
};

pub const U_IS_EMPTY: UnaryFunctionDefinition = UnaryFunctionDefinition {
    name: "isEmpty",
    function: eval_is_empty,
    validation: validate_unary_list,
    return_type: return_uni_boolean,
};

// Build a sorted-by-name table of unary function metas.
pub static UNARY_FUNCS: &[&UnaryFunctionDefinition] = &[
    &U_COUNT,
    &U_DATE,
    &U_DATETIME,
    &U_DAY_OF_WEEK,
    &U_DISTINCT_VALUES,
    &U_DUPLICATE_VALUES,
    &U_DURATION,
    &U_FLATTEN,
    &U_FROM_BASE64,
    &U_IS_EMPTY,
    &U_LAST_DAY_OF_MONTH,
    &U_LENGTH,
    &U_MAX,
    &U_MEAN,
    &U_MEDIAN,
    &U_MIN,
    &U_MODE,
    &U_MONTH_OF_YEAR,
    &U_PRODUCT,
    &U_REVERSE,
    &U_SANITIZE_FILENAME,
    &U_SORT,
    &U_SORT_DESCENDING,
    &U_STDDEV,
    &U_SUM,
    &U_TIME,
    &U_TO_BASE64,
    &U_TO_LOWER_CASE,
    &U_TO_STRING,
    &U_TO_UPPER_CASE,
    &U_TRIM,
];

#[inline(never)]
fn binary_search_by_name<'a, T>(table: &'a [&'a T], needle: &str, name_of: fn(&T) -> &str) -> Option<&'a T> {
    let (mut lo, mut hi) = (0usize, table.len());
    while lo < hi {
        let mid = (lo + hi) / 2;
        let mid_name = name_of(table[mid]);
        match mid_name.cmp(needle) {
            core::cmp::Ordering::Less => lo = mid + 1,
            core::cmp::Ordering::Greater => hi = mid,
            core::cmp::Ordering::Equal => return Some(table[mid]),
        }
    }
    None
}

#[inline]
pub fn lookup_unary(name: &str) -> Option<&'static UnaryFunctionDefinition> {
    binary_search_by_name(UNARY_FUNCS, name, |m| m.name)
}

pub const B_FIND: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "find",
    function: eval_find,
    validation: list_item_as_second_arg,
    return_type: return_binary_same_as_right_arg,
};

pub const B_CONTAINS: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "contains",
    function: eval_contains_mixed,
    validation: validate_binary_contains_mixed,
    return_type: return_boolean_type_binary,
};

pub const B_STARTS_WITH: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "startsWith",
    function: eval_starts_with,
    validation: validate_binary_string_string,
    return_type: return_boolean_type_binary,
};

pub const B_ENDS_WITH: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "endsWith",
    function: eval_ends_with,
    validation: validate_binary_string_string,
    return_type: return_boolean_type_binary,
};

pub const B_SPLIT: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "split",
    function: eval_split,
    validation: validate_binary_string_string,
    return_type: return_string_list_type_binary,
};

pub const B_REGEX_SPLIT: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "regexSplit",
    function: eval_regex_split,
    validation: validate_binary_string_string,
    return_type: return_string_list_type_binary,
};

pub const B_SUBSTRING_BEFORE: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "substringBefore",
    function: eval_substring_before,
    validation: validate_binary_string_string,
    return_type: return_string_type_binary,
};

pub const B_SUBSTRING_AFTER: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "substringAfter",
    function: eval_substring_after,
    validation: validate_binary_string_string,
    return_type: return_string_type_binary,
};

pub const B_CHAR_AT: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "charAt",
    function: eval_char_at,
    validation: validate_binary_string_number,
    return_type: return_string_type_binary,
};

pub const B_CHAR_CODE_AT: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "charCodeAt",
    function: eval_char_code_at,
    validation: validate_binary_string_number,
    return_type: return_number_type_binary,
};

pub const B_INDEX_OF: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "indexOf",
    function: eval_index_of_mixed,
    validation: validate_binary_index_of_mixed,
    return_type: return_index_of_type,
};

pub const B_LAST_INDEX_OF: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "lastIndexOf",
    function: eval_last_index_of,
    validation: validate_binary_string_string,
    return_type: return_number_type_binary,
};

pub const B_REPEAT: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "repeat",
    function: eval_repeat,
    validation: validate_binary_string_number,
    return_type: return_string_type_binary,
};

pub const B_INTERPOLATE: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "interpolate",
    function: eval_interpolate,
    validation: validate_binary_string_any,
    return_type: return_string_type_binary,
};

pub const B_REMOVE: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "remove",
    function: eval_remove,
    validation: validate_binary_list_number,
    return_type: return_binary_same_as_left_arg,
};

pub const B_PARTITION: BinaryFunctionDefinition = BinaryFunctionDefinition {
    name: "partition",
    function: eval_partition,
    validation: validate_binary_partition,
    return_type: super::function_list::return_partition_type,
};

// Sorted-by-name table of binary function metas
pub static BINARY_FUNCS: &[&BinaryFunctionDefinition] = &[
    &B_CHAR_AT,
    &B_CHAR_CODE_AT,
    &B_CONTAINS,
    &B_ENDS_WITH,
    &B_FIND,
    &B_INDEX_OF,
    &B_INTERPOLATE,
    &B_LAST_INDEX_OF,
    &B_PARTITION,
    &B_REGEX_SPLIT,
    &B_REMOVE,
    &B_REPEAT,
    &B_SPLIT,
    &B_STARTS_WITH,
    &B_SUBSTRING_AFTER,
    &B_SUBSTRING_BEFORE,
];

#[inline]
pub fn lookup_binary(name: &str) -> Option<&'static BinaryFunctionDefinition> {
    binary_search_by_name(BINARY_FUNCS, name, |m| m.name)
}

pub const M_MAX: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "max",
    function: eval_max_multi,
    validation: validate_multi_all_args_numbers,
    return_type: return_multi_number,
};

pub const M_SUM: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "sum",
    function: eval_sum_multi,
    validation: validate_multi_all_args_numbers,
    return_type: return_multi_number,
};

pub const M_MIN: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "min",
    function: eval_min_multi,
    validation: validate_multi_all_args_numbers,
    return_type: return_multi_number,
};

pub const M_SUBLIST: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "sublist",
    function: eval_sublist,
    validation: validate_multi_sublist,
    return_type: return_list_undefined,
};

pub const M_APPEND: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "append",
    function: eval_append,
    validation: validate_multi_append,
    return_type: return_list_undefined,
};

pub const M_CONCATENATE: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "concatenate",
    function: eval_concatenate,
    validation: validate_multi_concatenate,
    return_type: return_list_undefined,
};

pub const M_INSERT_BEFORE: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "insertBefore",
    function: eval_insert_before,
    validation: validate_multi_insert_before,
    return_type: return_list_undefined,
};

pub const M_UNION: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "union",
    function: eval_union,
    validation: validate_multi_union,
    return_type: return_list_undefined,
};

pub const M_JOIN: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "join",
    function: eval_join,
    validation: validate_multi_join,
    return_type: return_string_type_multi,
};

pub const M_SUBSTRING: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "substring",
    function: eval_substring,
    validation: validate_multi_substring,
    return_type: return_string_type_multi,
};

pub const M_REPLACE: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "replace",
    function: eval_replace,
    validation: validate_multi_replace,
    return_type: return_string_type_multi,
};

pub const M_REGEX_REPLACE: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "regexReplace",
    function: eval_regex_replace,
    validation: validate_multi_replace,
    return_type: return_string_type_multi,
};

pub const M_REPLACE_FIRST: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "replaceFirst",
    function: eval_replace_first,
    validation: validate_multi_replace,
    return_type: return_string_type_multi,
};

pub const M_REPLACE_LAST: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "replaceLast",
    function: eval_replace_last,
    validation: validate_multi_replace,
    return_type: return_string_type_multi,
};

pub const M_FROM_CHAR_CODE: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "fromCharCode",
    function: eval_from_char_code,
    validation: validate_multi_from_char_code,
    return_type: return_string_type_multi,
};

pub const M_PAD_START: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "padStart",
    function: eval_pad_start,
    validation: validate_multi_pad,
    return_type: return_string_type_multi,
};

pub const M_PAD_END: MultiFunctionDefinition = MultiFunctionDefinition {
    name: "padEnd",
    function: eval_pad_end,
    validation: validate_multi_pad,
    return_type: return_string_type_multi,
};

// Sorted-by-name table of multi-arg function metas
pub static MULTI_FUNCS: &[&MultiFunctionDefinition] = &[
    &M_APPEND,
    &M_CONCATENATE,
    &M_FROM_CHAR_CODE,
    &M_INSERT_BEFORE,
    &M_JOIN,
    &M_MAX,
    &M_MIN,
    &M_PAD_END,
    &M_PAD_START,
    &M_REGEX_REPLACE,
    &M_REPLACE,
    &M_REPLACE_FIRST,
    &M_REPLACE_LAST,
    &M_SUBLIST,
    &M_SUBSTRING,
    &M_SUM,
    &M_UNION,
];

#[inline]
pub fn lookup_multi(name: &str) -> Option<&'static MultiFunctionDefinition> {
    binary_search_by_name(MULTI_FUNCS, name, |m| m.name)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EFunctionType {
    Unary,
    Binary,
    Multi,
    Custom(u8),
}

#[inline]
pub fn lookup_built_in_function(name: &str) -> Option<EFunctionType> {
    if binary_search_by_name(UNARY_FUNCS, name, |m| m.name).is_some() {
        return Some(EFunctionType::Unary);
    }
    if binary_search_by_name(BINARY_FUNCS, name, |m| m.name).is_some() {
        return Some(EFunctionType::Binary);
    }
    if binary_search_by_name(MULTI_FUNCS, name, |m| m.name).is_some() {
        return Some(EFunctionType::Multi);
    }

    // Special reserved names that are not regular metas but should report as built-ins
    match name {
        "all" | "any" => Some(EFunctionType::Unary),
        _ => None,
    }
}

//--------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct BinaryFunction {
    pub left: ExpressionEnum,
    pub right: ExpressionEnum,
    pub definition: &'static BinaryFunctionDefinition,
    pub return_type: Link<ValueType>,
}

impl BinaryFunction {
    pub fn build(
        definition: &'static BinaryFunctionDefinition,
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
    pub definition: &'static UnaryFunctionDefinition,
    pub return_type: Link<ValueType>,
}

impl UnaryFunction {
    pub fn build(definition: &'static UnaryFunctionDefinition, arg: ExpressionEnum) -> Self {
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
    pub definition: &'static MultiFunctionDefinition,
    pub return_type: Link<ValueType>,
}

impl MultiFunction {
    pub fn build(definition: &'static MultiFunctionDefinition, args: Vec<ExpressionEnum>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unary_index_is_sorted() {
        assert!(UNARY_FUNCS
            .windows(2)
            .all(|w| w[0].name <= w[1].name));
    }

    #[test]
    fn binary_index_is_sorted() {
        assert!(BINARY_FUNCS
            .windows(2)
            .all(|w| w[0].name <= w[1].name));
    }

    #[test]
    fn multi_index_is_sorted() {
        assert!(MULTI_FUNCS
            .windows(2)
            .all(|w| w[0].name <= w[1].name));
    }
}
