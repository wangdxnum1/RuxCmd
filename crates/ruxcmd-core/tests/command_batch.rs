use ruxcmd_core::command_batch::{command_units, CommandBudget, MAX_COMMAND_UNITS};

#[test]
fn quotes_empty_backslashes_and_utf16_are_counted() {
    assert_eq!(command_units(&["x"]), 4); // quotes plus NUL
    assert_eq!(command_units(&["x", ""]), 7);
    assert_eq!(command_units(&["😀"]), 5);
    assert_eq!(command_units(&["a\\"]), 6);
    assert_eq!(command_units(&["a\"b"]), 7);
}

#[test]
fn prefix_and_exact_boundary_are_respected_and_reset() {
    let mut budget = CommandBudget::new(&["p"]).unwrap();
    let item = "a".repeat(MAX_COMMAND_UNITS - 7);
    assert!(budget.can_fit_item(&item));
    assert!(budget.try_add(&item));
    assert!(!budget.try_add(""));
    budget.reset();
    assert!(budget.try_add(&item));
    assert!(!budget.can_fit_item(&(item + "a")));
}

#[test]
fn oversized_fixed_command_is_rejected() {
    assert!(CommandBudget::new(&["a".repeat(MAX_COMMAND_UNITS)]).is_err());
}
