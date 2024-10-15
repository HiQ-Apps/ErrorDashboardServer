use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Role {
    Guest,
    Member,
    Contributor,
    Manager,
    Owner,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Permission {
    View,
    AddAlert,
    Comment,
    Delete,
    Invite,
    RemoveUser,
    Update,
}

#[derive(Debug, Clone)]
pub struct RolePermission {
    pub permissions: Vec<Permission>,
    pub weight: u8,
}

#[derive(Debug, Clone)]
pub struct RoleRules {
    pub role_rules: HashMap<Role, RolePermission>,
}

pub fn initialize_role_rules() -> RoleRules {
    let mut role_rules = HashMap::new();

    role_rules.insert(
        Role::Guest,
        RolePermission {
            permissions: vec![Permission::View],
            weight: 0,
        },
    );

    role_rules.insert(
        Role::Member,
        RolePermission {
            permissions: vec![Permission::View, Permission::Comment],
            weight: 1,
        },
    );

    role_rules.insert(
        Role::Contributor,
        RolePermission {
            permissions: vec![Permission::View, Permission::AddAlert, Permission::Comment],
            weight: 2,
        },
    );

    role_rules.insert(
        Role::Manager,
        RolePermission {
            permissions: vec![
                Permission::View,
                Permission::AddAlert,
                Permission::Comment,
                Permission::Delete,
                Permission::RemoveUser,
                Permission::Update,
            ],
            weight: 3,
        },
    );

    role_rules.insert(
        Role::Owner,
        RolePermission {
            permissions: vec![
                Permission::View,
                Permission::AddAlert,
                Permission::Comment,
                Permission::Delete,
                Permission::Invite,
                Permission::RemoveUser,
                Permission::Update,
            ],
            weight: 4,
        },
    );

    RoleRules { role_rules }
}

pub fn string_to_role(role_str: &str) -> Option<Role> {
    match role_str {
        "guest" => Some(Role::Guest),
        "member" => Some(Role::Member),
        "contributor" => Some(Role::Contributor),
        "manager" => Some(Role::Manager),
        "owner" => Some(Role::Owner),
        _ => None,
    }
}

pub fn get_perms<'a>(role_str: &str, role_rules: &'a RoleRules) -> Option<&'a RolePermission> {
    if let Some(role) = string_to_role(role_str) {
        return role_rules.role_rules.get(&role);
    }
    None
}

pub fn get_weight(role_str: &Role, role_rules: &RoleRules) -> Option<u8> {
    return role_rules.role_rules.get(role_str).map(|role| role.weight);
}
