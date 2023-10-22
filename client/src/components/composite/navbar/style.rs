#[allow(non_upper_case_globals)]
pub fn navbar_logo_container() -> String {
    stylist::style!(
        "
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        min-width: 300px;
        align-items: center;
        align-text: center;
        padding: 0 10px 0 10px;
        :hover {
            cursor: pointer;
        }
        @media screen and (max-width: 640px) {
            font-size: 14px;
        }
    "
    )
    .unwrap()
    .get_class_name()
    .to_string()
}

#[allow(non_upper_case_globals)]
pub fn menu_container() -> String {
    stylist::style!(
        "
        padding: 0 1% 0 0;
    "
    )
    .unwrap()
    .get_class_name()
    .to_string()
}

#[allow(non_upper_case_globals)]
pub fn navbar_container() -> String {
    stylist::style!(
        "
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        align-items: center;
        align-text: center;
        background-color: #DFE6DA;
    "
    )
    .unwrap()
    .get_class_name()
    .to_string()
}

#[allow(non_upper_case_globals)]
pub fn navlinks() -> String {
    stylist::style!(
        "
        display: flex;
        flex-direction: row;
        min-width: 250px;
        justify-content: space-around;
        @media screen and (max-width: 640px) {
            display: flex;
            position: absolute;
            justify-content: center;
            justify-items: center;
            flex-direction: column;
            min-width: unset;
            padding: unset;
            background-color: #d9dcd7;
            right: 1%;
            top: 60px;
        }
    "
    )
    .unwrap()
    .get_class_name()
    .to_string()
}
