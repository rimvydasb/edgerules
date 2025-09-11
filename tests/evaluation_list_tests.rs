mod utilities;
pub use utilities::*;

#[test]
fn list_membership_and_boolean_aggregates() {
    // contains with numbers, strings, booleans, dates
    assert_eq!(crate::eval_value("value : contains([1,2,3], 2)"), "true");
    assert_eq!(crate::eval_value("value : contains(['a','b','c'], 'd')"), "false");
    assert_eq!(crate::eval_value("value : contains([true,false], true)"), "true");
    assert_eq!(
        crate::eval_value("value : contains([date(\"2017-05-03\"), date(\"2017-05-04\")], date(\"2017-05-04\"))"),
        "true"
    );

    // all/any for booleans
    assert_eq!(crate::eval_value("value : all([true,true,false])"), "false");
    assert_eq!(crate::eval_value("value : any([false,false,true])"), "true");
}

#[test]
fn list_slicing_and_mutation() {
    // sublist
    assert_eq!(crate::eval_value("value : sublist([1,2,3], 2)"), "[2, 3]");
    assert_eq!(crate::eval_value("value : sublist([1,2,3], 1, 2)"), "[1, 2]");

    // append
    assert_eq!(crate::eval_value("value : append(['a'], 'b', 'c')"), "['a', 'b', 'c']");

    // concatenate
    assert_eq!(crate::eval_value("value : concatenate([1,2], [3])"), "[1, 2, 3]");

    // insertBefore (positions are 1-based)
    assert_eq!(crate::eval_value("value : insertBefore([1,3], 1, 2)"), "[2, 1, 3]");

    // remove at position (1-based)
    assert_eq!(crate::eval_value("value : remove([1,2,3], 2)"), "[1, 3]");
}

#[test]
fn list_order_and_indexing() {
    // reverse (list)
    assert_eq!(crate::eval_value("value : reverse([1,2,3])"), "[3, 2, 1]");
    assert_eq!(crate::eval_value("value : reverse(['a','b','c'])"), "['c', 'b', 'a']");

    // indexOf returns 1-based positions (list)
    assert_eq!(crate::eval_value("value : indexOf([1,2,3,2], 2)"), "[2, 4]");

    // sort default ascending (second argument ignored for now)
    assert_eq!(crate::eval_value("value : sort([3,1,4,2], 0)"), "[1, 2, 3, 4]");
    assert_eq!(crate::eval_value("value : sort(['b','a','c'], 0)"), "['a', 'b', 'c']");
}

#[test]
fn list_set_ops_and_flatten() {
    // union (dedup across lists)
    assert_eq!(crate::eval_value("value : union([1,2], [2,3])"), "[1, 2, 3]");

    // distinct / duplicates
    assert_eq!(crate::eval_value("value : distinctValues([1,2,3,2,1])"), "[1, 2, 3]");
    assert_eq!(crate::eval_value("value : duplicateValues([1,2,3,2,1])"), "[2, 1]");

    // flatten
    assert_eq!(crate::eval_value("value : flatten([[1,2], [[3]], 4])"), "[1, 2, 3, 4]");
}

#[test]
fn list_join_empty_partition() {
    // join variants
    assert_eq!(
        crate::eval_value("value : join(['a','b','c'], ', ')"),
        "'a, b, c'"
    );
    assert_eq!(
        crate::eval_value("value : join(['a','b','c'], ', ', '[', ']')"),
        "'[a, b, c]'"
    );

    // isEmpty
    assert_eq!(crate::eval_value("value : isEmpty(sublist([1], 1, 0))"), "true");
    assert_eq!(crate::eval_value("value : isEmpty([1])"), "false");

    // partition
    assert_eq!(
        crate::eval_value("value : partition([1,2,3,4,5], 2)"),
        "[[1, 2], [3, 4], [5]]"
    );
}

#[test]
fn list_numeric_aggregates() {
    assert_eq!(crate::eval_value("value : min([1,2,3])"), "1");
    assert_eq!(crate::eval_value("value : product([2,3,4])"), "24");
    assert_eq!(crate::eval_value("value : mean([1,2,3])"), "2");
    assert_eq!(crate::eval_value("value : median([1,2,3])"), "2");
    assert_eq!(crate::eval_value("value : stddev([2,4])"), "1");
    assert_eq!(crate::eval_value("value : mode([1,2,2,3])"), "[2]");
}
