mod utilities;
pub use utilities::*;

#[test]
fn list_membership_and_boolean_aggregates() {
    // contains with numbers, strings, booleans, dates
    assert_value!("contains([1,2,3], 2)", "true");
    assert_value!("contains(['a','b','c'], 'd')", "false");
    assert_value!("contains(['ass','bss','css'], 'bss')", "true");
    assert_value!(
        r#"
        provided: ['ass','bss','css']
        value: contains(provided, 'bss')
    "#,
        "true"
    );
    assert_value!("contains([true,false], true)", "true");
    assert_value!(
        "contains([date('2017-05-03'), date('2017-05-04')], date('2017-05-04'))",
        "true"
    );

    // all/any for booleans
    assert_value!("all([true,true,false])", "false");
    assert_value!("any([false,false,true])", "true");
    assert_value!(
        r#"
        numbers: [1, 2, 3]
        value: all(for n in numbers return n < 4)
    "#,
        "true"
    );
    assert_value!(
        r#"
        numbers: [1, 2, 3]
        threshold: 2
        value: any(for n in numbers return n > threshold)
    "#,
        "true"
    );
}

#[test]
fn list_slicing_and_mutation() {
    // sublist
    assert_value!("sublist([1,2,3], 2)", "[2, 3]");
    assert_value!("sublist([1,2,3], 1, 2)", "[1, 2]");

    // append
    assert_value!("append(['a'], 'b', 'c')", "['a', 'b', 'c']");

    // concatenate
    assert_value!("concatenate([1,2], [3])", "[1, 2, 3]");

    // insertBefore (positions are 1-based)
    assert_value!("insertBefore([1,3], 1, 2)", "[2, 1, 3]");

    // remove at position (1-based)
    assert_value!("remove([1,2,3], 2)", "[1, 3]");
}

#[test]
fn list_order_and_indexing() {
    // reverse (list)
    assert_value!("reverse([1,2,3])", "[3, 2, 1]");
    assert_eq!(
        crate::eval_value("value : reverse(['a','b','c'])"),
        "['c', 'b', 'a']"
    );

    // indexOf returns 1-based positions (list)
    assert_value!("indexOf([1,2,3,2], 2)", "[2, 4]");

    // sort default ascending
    assert_value!("sort([3,1,4,2])", "[1, 2, 3, 4]");
    assert_value!("sort(['b','a','c'])", "['a', 'b', 'c']");

    assert_value!("sortDescending([3,1,4,2])", "[4, 3, 2, 1]");
    assert_value!("sortDescending(['b','a','c'])", "['c', 'b', 'a']");
}

#[test]
fn list_set_ops_and_flatten() {
    // union (dedup across lists)
    assert_value!("union([1,2], [2,3])", "[1, 2, 3]");

    // distinct / duplicates
    assert_value!("distinctValues([1,2,3,2,1])", "[1, 2, 3]");
    assert_value!("duplicateValues([1,2,3,2,1])", "[2, 1]");

    // flatten
    // @Todo: as of now only homogeneous lists are supported, so it is unclear how flattening should happen
    //assert_value!("flatten([[1,2], [[3]], 4])", "[1, 2, 3, 4]");
}

#[test]
fn list_join_empty_partition() {
    // join variants
    assert_value!("join(['a','b','c'], ', ')", "'a, b, c'");
    assert_value!("join(['a','b','c'], ', ', '[', ']')", "'[a, b, c]'");

    // isEmpty
    assert_value!("isEmpty(sublist([1], 1, 0))", "true");
    assert_value!("isEmpty([1])", "false");

    // partition
    assert_value!("partition([1,2,3,4,5], 2)", "[[1, 2], [3, 4], [5]]");
}

#[test]
fn list_numeric_aggregates() {
    assert_value!("min([1,2,3])", "1");
    assert_value!("product([2,3,4])", "24");
    assert_value!("mean([1,2,3])", "2");
    assert_value!("median([1,2,3])", "2");
    assert_value!("stddev([2,4])", "1");
    assert_value!("mode([1,2,2,3])", "[2]");
}

#[test]
fn list_extrema_temporal_values() {
    assert_value!(
        "max([date('2020-01-01'), date('2020-05-01')])",
        "2020-05-01"
    );
    assert_value!("min([time('10:00:00'), time('08:00:00')])", "8:00:00.0");
    assert_value!(
        "max([datetime('2020-01-01T00:00:00'), datetime('2020-01-02T03:00:00')])",
        "2020-01-023:00:00.0"
    );
    assert_value!("min([duration('P1D'), duration('P2D')])", "P1D");
    assert_value!("max([duration('P1M'), duration('P2M')])", "P2M");
    assert_value!("max(date('2020-01-01'), date('2020-05-01'))", "2020-05-01");
    assert_value!("min(duration('P1D'), duration('P2D'))", "P1D");
}

#[test]
fn complex_objects_in_lists() {
    assert_value!(
        r#"
        list: [{a:1}, {a:2}, {a:3}]
        value: for item in list return item.a
    "#,
        "[1, 2, 3]"
    );

    assert_value!(
        r#"
        config: 5
        list: [{a:1 + config}, {a:2 + config}, {a:3 + config}]
        value: for item in list return item.a
    "#,
        "[6, 7, 8]"
    );
}

#[test]
fn list_numeric_unhappy_paths() {
    // Using strings where numbers are expected
    link_error_contains("value : product(['a','b'])", &["unexpected", "number"]);
    link_error_contains("value : mean(['1','2'])", &["unexpected", "number"]);
    link_error_contains("value : median(['x'])", &["unexpected", "number"]);
    link_error_contains("value : stddev(['x','y'])", &["unexpected", "number"]);

    // Using dates where numbers are expected
    link_error_contains(
        "value : product([date('2017-05-03')])",
        &["unexpected", "number"],
    );
    link_error_contains(
        "value : mean([date('2017-05-03'), date('2017-05-04')])",
        &["unexpected", "number"],
    );
}
