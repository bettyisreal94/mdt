use boa_engine::{
    object::builtins::{JsArray, JsFunction},
    Context, JsObject, JsString, JsValue, Source,
};
use std::collections::HashMap;

/// Parse a JavaScript function expression into a Rust rule.
///
/// The input string must be a function expression (e.g. `(t, r, i) => ...` or
/// `function (t, r, i) { ... }`). The returned [`JsRule`] can be invoked per row
/// via [`JsRule::apply`], which exposes the arguments as:
///
/// - `t` (table): the whole table, as a JavaScript array of row objects.
/// - `r` (row): the current row, as a JavaScript object keyed by column name.
/// - `i` (index): the zero-based index of the current row.
///
/// The result is coerced to a string, matching the way a cell value is rendered
/// in the extended markdown table format.
pub fn str_to_fn(code: &str) -> Result<JsRule, String> {
    let mut context = Context::default();

    let wrapped = format!("(function() {{ return {code}; }})()");
    let value = context
        .eval(Source::from_bytes(&wrapped))
        .map_err(|e| format!("Failed to parse function: {e}"))?;

    if !value.is_callable() {
        return Err(format!("Input is not a function expression: {code}"));
    }

    let obj = value
        .as_object()
        .ok_or_else(|| format!("Input did not evaluate to an object: {code}"))?;
    let func = JsFunction::from_object(obj)
        .ok_or_else(|| format!("Input did not evaluate to a function: {code}"))?;

    Ok(JsRule {
        func: func.into(),
        source: code.to_string(),
    })
}

/// A parsed JavaScript column rule, ready to be invoked per row.
#[derive(Clone)]
pub struct JsRule {
    func: JsObject,
    source: String,
}

impl JsRule {
    /// Invoke the rule for a single row and return the coerced cell value.
    pub fn apply(
        &self,
        context: &mut Context,
        table: &[HashMap<String, String>],
        row: &HashMap<String, String>,
        index: usize,
    ) -> Result<String, String> {
        let table_obj = build_table(context, table);
        let row_obj = build_row(context, row);
        let args: Vec<JsValue> = vec![table_obj, row_obj, JsValue::from(index as f64)];

        let result = self
            .func
            .call(&JsValue::undefined(), &args, context)
            .map_err(|e| format!("Error evaluating rule: {e}"))?;

        to_cell_string(&result, context)
    }

    /// The original source code of this rule.
    pub fn source(&self) -> &str {
        &self.source
    }
}

fn build_table(context: &mut Context, table: &[HashMap<String, String>]) -> JsValue {
    let array = JsArray::new(context).expect("array construction");
    for row in table {
        array
            .push(build_row(context, row), context)
            .expect("array push");
    }
    array.into()
}

fn build_row(context: &mut Context, row: &HashMap<String, String>) -> JsValue {
    let obj = JsObject::with_object_proto(context.intrinsics());
    for (key, value) in row {
        obj.set(
            JsString::from(key.clone()),
            JsValue::from(JsString::from(value.as_str())),
            false,
            context,
        )
        .expect("object property set");
    }
    obj.into()
}

fn to_cell_string(value: &JsValue, context: &mut Context) -> Result<String, String> {
    let result = value
        .to_string(context)
        .map_err(|e| format!("Failed to coerce result to string: {e}"))?;
    result.to_std_string().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_table() -> Vec<HashMap<String, String>> {
        let mut table = Vec::new();
        for (name, age, job) in [
            ("Mira", "27", "Superhero"),
            ("Pietro", "24", "Clown"),
            ("Flora", "18", "Singer"),
        ] {
            let mut row = HashMap::new();
            row.insert("name".to_string(), name.to_string());
            row.insert("age".to_string(), age.to_string());
            row.insert("job".to_string(), job.to_string());
            table.push(row);
        }
        table
    }

    #[test]
    fn test_parse_failure() {
        let err = match str_to_fn("this is not (valid javascript") {
            Ok(_) => panic!("expected a parse error"),
            Err(err) => err,
        };
        assert!(
            err.contains("Failed to parse function"),
            "expected a parse error, got: {err}"
        );
    }

    #[test]
    fn test_parse_non_function_expression() {
        let err = match str_to_fn("42") {
            Ok(_) => panic!("expected a non-function error"),
            Err(err) => err,
        };
        assert!(
            err.contains("not a function"),
            "expected a non-function error, got: {err}"
        );
    }

    #[test]
    fn test_parse_and_use_arrow_function() {
        let rule = str_to_fn(r#"(t, r, i) => `${r["name"]}, the ${r["job"]}`"#)
            .expect("Failed to parse arrow function");

        let mut context = Context::default();
        let table = sample_table();

        let out = rule
            .apply(&mut context, &table, &table[0], 0)
            .expect("Failed to apply rule");
        assert_eq!(out, "Mira, the Superhero");

        let out = rule
            .apply(&mut context, &table, &table[2], 2)
            .expect("Failed to apply rule");
        assert_eq!(out, "Flora, the Singer");
    }

    #[test]
    fn test_parse_and_use_function_declaration() {
        let rule = str_to_fn(
            r#"function (t, r, i) { return r["name"] + " is " + r["age"] + " years old"; }"#,
        )
        .expect("Failed to parse function expression");

        let mut context = Context::default();
        let table = sample_table();

        let out = rule
            .apply(&mut context, &table, &table[1], 1)
            .expect("Failed to apply rule");
        assert_eq!(out, "Pietro is 24 years old");
    }

    #[test]
    fn test_rule_uses_index_argument() {
        let rule = str_to_fn("(t, r, i) => i + 1")
            .expect("Failed to parse function");

        let mut context = Context::default();
        let table = sample_table();

        let out = rule
            .apply(&mut context, &table, &table[0], 2)
            .expect("Failed to apply rule");
        assert_eq!(out, "3");
    }

    #[test]
    fn test_rule_accesses_whole_table() {
        let rule = str_to_fn("(t, r, i) => t.length")
            .expect("Failed to parse function");

        let mut context = Context::default();
        let table = sample_table();

        let out = rule
            .apply(&mut context, &table, &table[0], 0)
            .expect("Failed to apply rule");
        assert_eq!(out, "3");
    }
}
