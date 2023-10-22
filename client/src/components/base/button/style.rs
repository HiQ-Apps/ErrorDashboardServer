#[allow(non_upper_case_globals)]
pub fn button_container() -> String {
    stylist::style!(
        "
    "
    )
    .unwrap()
    .get_class_name()
    .to_string()
}

#[allow(non_upper_case_globals)]
pub fn button_text() -> String {
    stylist::style!(
        "
    "
    )
    .unwrap()
    .get_class_name()
    .to_string()
}
