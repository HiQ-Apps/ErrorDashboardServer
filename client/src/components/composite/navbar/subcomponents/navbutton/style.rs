#[allow(non_upper_case_globals)]
pub fn menubutton() -> String {
    stylist::style!(
        "
        display: none;
        background-color: #d9dcd7;
        justify-content: space-around;
        justify-items: center;
        padding: 2px 4px 0px 4px;
        min-width: 30px;
        min-height: 30px;
        font-size: 1rem;
        border: 1px black solid;
        border-radius: 5px;
        :hover {
            background-color: #babfb6;
            cursor: pointer;
        }
        @media screen and (max-width: 641px) {
            display: flex;
        }
    "
    )
    .unwrap()
    .get_class_name()
    .to_string()
}

#[allow(non_upper_case_globals)]
pub fn navbutton() -> String {
    stylist::style!(
        "
        display: flex;
        background-color: #d9dcd7;
        padding: 2px 4px 0px 4px;
        min-width: 30px;
        min-height: 30px;
        font-size: 1rem;
        border: none;
        border-radius: 5px;
        :hover {
            background-color: #babfb6;
            cursor: pointer;
        }
        @media screen and (max-width: 640px) {
            display: none;
        }
    "
    )
    .unwrap()
    .get_class_name()
    .to_string()
}

#[allow(non_upper_case_globals)]
pub fn dropdown_navbutton() -> String {
    stylist::style!(
        "
        display: flex;
        background-color: #d9dcd7;
        justify-content: space-around;
        justify-items: center;
        padding: 2px 4px 0px 4px;
        min-width: 30px;
        min-height: 30px;
        font-size: 1rem;
        border: none;
        border-radius: 5px;
        :hover {
            background-color: #babfb6;
            cursor: pointer;
        }
    "
    )
    .unwrap()
    .get_class_name()
    .to_string()
}
