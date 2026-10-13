//! Common-domain requests, through each answering entrypoint
//! (§REQ-one-answer.proof.matrix). Paths retain explicit/implicit invocation.

use serde_json::{Value, json};

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

    pub fn python_request(&self, cwd: &str) -> Value {
        json!({"operation":self.operation,"root":self.path,"explicit":self.explicit,
            "cwd":cwd,"args":self.args,"options":self.options})
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
        json!({"operation":self.node_operation,"cwd":cwd,"args":args})
    }
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
