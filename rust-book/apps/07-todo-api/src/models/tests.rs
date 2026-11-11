use serde_json::{Value, json};

use crate::models::{CreateTodo, UpdateTodo};

#[test]
fn create_trims_the_title() {
    assert_eq!(
        CreateTodo::from_json(&json!({ "title": "  write docs  " })),
        Ok(CreateTodo {
            title: "write docs".into()
        })
    );
}

#[test]
fn create_reports_every_problem() {
    let long = "x".repeat(201);
    let cases: [(Value, &str); 7] = [
        (json!({}), "title: required"),
        (json!({ "title": "" }), "title: must not be empty"),
        (json!({ "title": "   " }), "title: must not be empty"),
        (
            json!({ "title": long }),
            "title: must be at most 200 characters",
        ),
        (json!({ "title": 5 }), "title: expected a string"),
        (
            json!({ "title": "ok", "extra": 1 }),
            "(body): unrecognized key 'extra'",
        ),
        (json!([]), "(body): expected an object"),
    ];
    for (body, issue) in cases {
        assert_eq!(
            CreateTodo::from_json(&body).unwrap_err()[0],
            issue,
            "{body}"
        );
    }
}

#[test]
fn update_accepts_any_subset() {
    assert_eq!(
        UpdateTodo::from_json(&json!({ "done": true })),
        Ok(UpdateTodo {
            title: None,
            done: Some(true)
        })
    );
    assert_eq!(
        UpdateTodo::from_json(&json!({ "title": " b ", "done": false })),
        Ok(UpdateTodo {
            title: Some("b".into()),
            done: Some(false)
        })
    );
}

#[test]
fn update_rejects_bad_bodies() {
    let cases: [(Value, &str); 6] = [
        (json!({}), "(body): provide at least one of: title, done"),
        (json!({ "done": "yes" }), "done: expected a boolean"),
        (json!({ "done": null }), "done: expected a boolean"),
        (json!({ "title": "" }), "title: must not be empty"),
        (json!({ "nope": 1 }), "(body): unrecognized key 'nope'"),
        (json!(3), "(body): expected an object"),
    ];
    for (body, issue) in cases {
        assert_eq!(
            UpdateTodo::from_json(&body).unwrap_err()[0],
            issue,
            "{body}"
        );
    }
}
