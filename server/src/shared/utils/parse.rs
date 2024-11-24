#[derive(Debug, Clone)]
pub struct StackTraceInfo {
    pub error_type: String,
    pub error_message: String,
    pub file_path: String,
    pub line_number: i32,
    pub function_name: String,
    pub module_name: String,
    pub stack_trace_lines: Vec<String>,
}

impl StackTraceInfo {
    pub fn default() -> Self {
        StackTraceInfo {
            error_type: String::new(),
            error_message: String::new(),
            file_path: String::new(),
            line_number: 0,
            function_name: String::new(),
            module_name: String::new(),
            stack_trace_lines: Vec::new(),
        }
    }

    pub fn new(
        error_type: String,
        error_message: String,
        file_path: String,
        line_number: i32,
        function_name: String,
        module_name: String,
        stack_trace_lines: Vec<String>,
    ) -> Self {
        StackTraceInfo {
            error_type,
            error_message,
            file_path,
            line_number,
            function_name,
            module_name,
            stack_trace_lines,
        }
    }
}


pub fn parse_stack_trace(stack_trace: &String) -> Result<StackTraceInfo, &'static str> {
    let lines: Vec<&str> = stack_trace.lines().collect();

    if lines.is_empty() {
        return Err("Stack trace is empty");
    }

    let mut error_type = String::new();
    let mut error_message = String::new();
    let mut file_path = String::new();
    let mut line_number = 0;
    let mut function_name = String::new();
    let mut module_name = String::new();
    let mut stack_trace_lines = Vec::new();

    if !lines.is_empty() {
        let first_line_parts: Vec<&str> = lines[0].splitn(2, ": ").collect();
        if first_line_parts.len() > 1 {
            error_type = first_line_parts[0].to_string();
            error_message = first_line_parts[1].to_string();
        } else {
            error_message = lines[0].to_string();
        }
    }

    for line in lines.iter().skip(1) {
        stack_trace_lines.push(line.to_string());
        let line_parts: Vec<&str> = line.split_whitespace().collect();

        if line_parts.len() > 1 {
            let error_origin = line_parts[1];
            let error_origin_parts: Vec<&str> = error_origin.split(':').collect();

            if error_origin_parts.len() > 1 {
                file_path = error_origin_parts[0].to_string();
                line_number = error_origin_parts[1].parse().unwrap();
            }

            if line_parts.len() > 2 {
                function_name = line_parts[2].to_string();
            }

            if line_parts.len() > 3 {
                module_name = line_parts[3].to_string();
            }
        }
    }

    Ok(StackTraceInfo::new(
        error_type,
        error_message,
        file_path,
        line_number,
        function_name,
        module_name,
        stack_trace_lines,
    ))
}

