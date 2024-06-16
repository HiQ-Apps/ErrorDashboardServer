// Build query string for database operations
pub fn create_query(action: &str, table:&str, condition:Option<&str>, sort_by: Option<&str>, group_by: Option<&str>) -> String {
    let mut query = format!("SELECT {} FROM {} ", action, table);

    if let Some(condition) = condition {
        query.push_str(format!("WHERE {}", condition).as_str());
    };
    if let Some(sort_by) = sort_by {
        query.push_str(format!("ORDER BY {}", sort_by).as_str());
    };
    if let Some(group_by) = group_by {
        query.push_str(format!("GROUP BY {}", group_by).as_str());
    };
    query
}

