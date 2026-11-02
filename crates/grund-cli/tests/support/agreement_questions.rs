//! Common-domain requests, through each answering entrypoint
//! (§REQ-one-answer.proof.matrix). Paths retain explicit/implicit invocation.

use serde_json::{Map, Value, json};

#[derive(Clone)]
pub struct Question {
    pub surface: &'static str,
    pub operation: &'static str,
    pub node_operation: &'static str,
    pub cli: Vec<String>,
    pub args: Vec<Value>,
    pub options: Value,
    pub path: String,
    pub explicit: bool,
    pub point: &'static str,
    pub scopes: Value,
}

impl Question {
    fn new(
        context: &Value,
        surface: &'static str,
        operation: &'static str,
        node_operation: &'static str,
        cli: &[&str],
        point: &'static str,
    ) -> Self {
        Self {
            surface,
            operation,
            node_operation,
            cli: cli.iter().map(|s| (*s).into()).collect(),
            args: Vec::new(),
            options: json!({}),
            path: context["path"].as_str().unwrap().into(),
            explicit: context["explicit"].as_bool().unwrap(),
            point,
            scopes: context["scopes"].clone(),
        }
    }

    pub fn cli_args(&self) -> Vec<String> {
        let mut args = self.cli.clone();
        if self.explicit {
            args.push(self.path.clone());
        }
        if self.surface != "fmt" {
            args.extend(["--format".into(), "json".into()]);
        }
        args
    }

    /// The context `surface` answers this question in, under that surface's
    /// scope. Of the variant's authored truth it carries the calls that judge
    /// this question, its own and the editor's, rather than a copy of every call
    /// of the variant per question (§REQ-one-answer.proof.delivery.payment); a
    /// call it lacks still fails `agreement_truth::attach`'s preflight
    /// (§REQ-one-answer.proof.evidence).
    pub fn context(&self, context: &Value, surface: &str) -> Value {
        let mut scoped = invocation(context);
        match context.get("truth") {
            Some(Value::Object(truth)) => {
                let key = super::agreement_truth::key(self);
                let calls: Map<String, Value> = [format!("editor:{key}"), key]
                    .into_iter()
                    .filter_map(|key| Some((key.clone(), truth.get("calls")?.get(&key)?.clone())))
                    .collect();
                let mut own: Map<String, Value> = truth
                    .iter()
                    .filter(|(name, _)| *name != "calls")
                    .map(|(name, value)| (name.clone(), value.clone()))
                    .collect();
                own.insert("calls".into(), Value::Object(calls));
                scoped.insert("truth".into(), Value::Object(own));
            }
            Some(truth) => {
                scoped.insert("truth".into(), truth.clone());
            }
            None => {}
        }
        let mut scoped = Value::Object(scoped);
        if let Some(scope) = self.scopes[surface].as_object() {
            for (key, value) in scope {
                scoped[key] = value.clone();
            }
        }
        scoped
    }

    pub fn python_request(&self, cwd: &str) -> Value {
        json!({"operation":self.operation,"root":self.path,"explicit":self.explicit,
            "cwd":cwd,"args":self.args,"options":self.options,"scope":self.scopes["python"]})
    }

    pub fn node_request(&self, cwd: &str) -> Value {
        let mut options = self.options.clone();
        if self.explicit {
            options["root"] = json!(self.path);
        }
        let args = match self.surface {
            "check" => {
                if self.explicit {
                    vec![json!(self.path)]
                } else {
                    Vec::new()
                }
            }
            "show-batch" => vec![Value::Null, options],
            "show" | "refs" => vec![self.args[0].clone(), options],
            _ => vec![options],
        };
        json!({"operation":self.node_operation,"cwd":cwd,"args":args,"scope":self.scopes["node"]})
    }
}

/// A variant's invocation facts, its authored truth left out: all that a scope
/// question or a question's context takes from it.
pub fn invocation(context: &Value) -> Map<String, Value> {
    context
        .as_object()
        .unwrap()
        .iter()
        .filter(|(name, _)| *name != "truth")
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

pub fn tree_questions(context: &Value) -> Vec<Question> {
    let mut questions = vec![
        Question::new(
            context,
            "check",
            "check",
            "check",
            &["check"],
            "§FS-check.1.1",
        ),
        Question::new(
            context,
            "list",
            "list_ids",
            "list",
            &["list"],
            "§REQ-one-answer.identity.declarations",
        ),
        Question::new(
            context,
            "list-size",
            "list_sizes",
            "listSizes",
            &["list", "--size"],
            "§REQ-one-answer.sections.map",
        ),
        Question::new(
            context,
            "cover",
            "cover",
            "cover",
            &["cover"],
            "§FS-scan.path.cover",
        ),
        Question::new(
            context,
            "cover-lines",
            "cover",
            "cover",
            &["cover", "--lines", "1"],
            "§FS-cover.6.1",
        ),
        Question::new(
            context,
            "fmt",
            "fmt",
            "fmt",
            &["fmt"],
            "§REQ-one-answer.writes.anchors",
        ),
        Question::new(
            context,
            "show-batch",
            "show_batch",
            "showBatch",
            &["show", "--batch", "--all", "--toc"],
            "§REQ-one-answer.sections.map",
        ),
    ];
    let selected = std::path::Path::new(context["path"].as_str().unwrap());
    let saved = std::path::Path::new(context["saved"].as_str().unwrap());
    let file = context["overlayRelative"].as_str().or_else(|| {
        context["textFiles"]
            .as_array()
            .unwrap()
            .iter()
            .chain(context["files"].as_array().unwrap())
            .filter_map(Value::as_str)
            .find(|file| saved.join(file).starts_with(selected))
    });
    let lines = &mut questions[4];
    if let Some(file) = file {
        lines.path = std::path::Path::new(context["saved"].as_str().unwrap())
            .join(file)
            .to_string_lossy()
            .into_owned();
    }
    lines.explicit = true;
    lines.options = json!({"lines":["1"]});
    questions[5].options = json!({"write":false});
    questions[6].options = json!({"mode":"toc"});
    questions
}

/// Each declared coordinate gets both queries; empty/refused catalogs still ask
/// an absent coordinate, whose refusal is an answer (§REQ-one-answer.proof.matrix).
pub fn coordinate_questions(context: &Value, coordinate: &str) -> Vec<Question> {
    let mut show = Question::new(
        context,
        "show",
        "show",
        "show",
        &["show", coordinate, "--toc"],
        "§REQ-one-answer.sections.map",
    );
    show.args = vec![json!(coordinate)];
    show.options = json!({"mode":"toc","format":"json"});
    let mut refs = Question::new(
        context,
        "refs",
        "refs",
        "refs",
        &["refs", coordinate],
        "§FS-refs.2",
    );
    refs.args = vec![json!(coordinate)];
    vec![show, refs]
}
