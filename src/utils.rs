use cli_table::format::{Border, Separator};
use cli_table::{print_stdout, Cell, CellStruct, Style, Table, TableStruct};
use roxmltree::Document;
use rusoto_core::RusotoError;

pub fn name_query(query: &Option<String>, exact_q: &Option<String>) -> Option<Vec<String>> {
    let input = query.as_ref().map(|q| split(q, false));
    let exact_input = exact_q.as_ref().map(|q| split(q, true));
    if input.is_none() && exact_input.is_none() {
        None
    } else {
        Some([input.unwrap_or_default(), exact_input.unwrap_or_default()].concat())
    }
}
pub fn split(q: &str, is_exact: bool) -> Vec<String> {
    let format = |s: &str| {
        if is_exact {
            s.to_string()
        } else {
            format!("*{}*", s)
        }
    };
    q.split(',').map(|s| format(s)).collect()
}

#[test]
fn test_name_query() {
    assert_eq!(
        name_query(&Some("test".to_string()), &None),
        Some(vec!["*test*".to_string()])
    );
    assert_eq!(
        name_query(&Some("api,test".to_string()), &None),
        Some(vec!["*api*".to_string(), "*test*".to_string()])
    );
    assert_eq!(
        name_query(&None, &Some("api".to_string())),
        Some(vec!["api".to_string()])
    );
}

pub fn err_handler<E>(error: RusotoError<E>) -> String {
    match error {
        RusotoError::Unknown(ref e) => {
            let doc = Document::parse(e.body_as_str()).unwrap();
            let finder = |s: &str| {
                doc.descendants()
                    .find(|n| n.has_tag_name(s))
                    .map(|n| n.text())
                    .flatten()
                    .unwrap_or("unknown")
            };
            format!(
                "[ERROR] code:{}, message: {}",
                finder("Code"),
                finder("Message")
            )
        }
        _ => "[ERROR] unknown error".to_string(),
    }
}

pub fn print_table(header: Vec<String>, rows: Vec<Vec<String>>) {
    let h: Vec<CellStruct> = header.iter().map(|h| h.cell().bold(true)).collect();
    let rows: Vec<Vec<CellStruct>> = rows
        .iter()
        .map(|r| r.iter().map(|c| c.cell()).collect())
        .collect();

    let t: TableStruct = rows
        .table()
        .title(h)
        .border(Border::builder().build())
        .separator(Separator::builder().build());
    match print_stdout(t) {
        Ok(r) => r,
        Err(e) => panic!("{:?}", e),
    };
}

pub struct Tag {
    pub key: String,
    pub value: Option<String>,
}

// find tag values
pub fn get_values(tags: &[Tag], keys: &[String]) -> Vec<String> {
    let mut result: Vec<String> = vec!["".to_string(); keys.len()];
    for tag in tags {
        let index = keys.iter().position(|k| k.eq(&tag.key));
        index
            .into_iter()
            .for_each(|i| result[i] = tag.value.as_ref().cloned().unwrap_or_default())
    }
    result
}

#[test]
fn test_get_values() {
    let tags = vec![
        Tag {
            key: "Name".to_string(),
            value: Some("api".to_string()),
        },
        Tag {
            key: "Env".to_string(),
            value: Some("staging".to_string()),
        },
        Tag {
            key: "aws:autoscaling:groupName".to_string(),
            value: Some("spot-api".to_string()),
        },
        Tag {
            key: "ignore-monitor".to_string(),
            value: None,
        },
    ];
    assert_eq!(
        get_values(&tags, &vec!["Name".to_string()]),
        vec!["api".to_string()],
    );
    assert_eq!(
        get_values(&tags, &vec!["Env".to_string()]),
        vec!["staging".to_string()],
    );
    assert_eq!(
        get_values(&tags, &vec!["Env".to_string(), "Name".to_string()]),
        vec!["staging".to_string(), "api".to_string()]
    );
    assert_eq!(get_values(&tags, &vec!["env".to_string()]), vec![""]);
    assert_eq!(
        get_values(&tags, &vec!["ignore-monitor".to_string()]),
        vec![""]
    );
}
