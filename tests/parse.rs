use gomod_parser::GoMod;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;

fn get_test_file_path(file_name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.join(format!("tests/data/{file_name}"))
}

fn get_test_file_content(file_name: &str) -> String {
    let path = get_test_file_path(file_name);
    read_to_string(path).unwrap()
}

#[test]
fn test_parse_on_compress() {
    let file_content = get_test_file_content("compress.mod");
    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(gomod.module, "github.com/klauspost/compress".to_string());
}

#[test]
fn test_parse_on_docker_docs() {
    let file_content = get_test_file_content("docker_docs.mod");
    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(gomod.module, "github.com/docker/docs".to_string());
}

#[test]
fn test_from_str_on_docker_docs() {
    let file_content = get_test_file_content("docker_docs.mod");
    let gomod = GoMod::from_str(&file_content).unwrap();

    assert_eq!(gomod.module, "github.com/docker/docs".to_string());
}

#[test]
fn test_parse_on_iris() {
    let file_content = get_test_file_content("iris.mod");
    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(gomod.module, "github.com/kataras/iris/v12".to_string());
}

#[test]
fn test_parse_on_kubernetes() {
    let file_content = get_test_file_content("kubernetes.mod");
    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(gomod.module, "k8s.io/kubernetes".to_string());
}

#[test]
fn test_parse_on_prometheus() {
    let file_content = get_test_file_content("prometheus.mod");
    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(gomod.module, "github.com/prometheus/prometheus".to_string());
}

#[test]
fn test_parse_godebug() {
    let file_content = get_test_file_content("godebug.mod");
    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(
        gomod.godebug,
        HashMap::from_iter([
            ("asynctimerchan".into(), "0".into()),
            ("default".into(), "go1.21".into()),
            ("panicnil".into(), "1".into()),
        ])
    );
}

#[test]
fn test_parse_tool() {
    let file_content = get_test_file_content("tool.mod");
    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(
        gomod.tool,
        vec![
            "example.com/mymodule/cmd/mytool1",
            "example.com/mymodule/cmd/mytool2"
        ]
    );
}

#[test]
fn test_carriage_return() {
    let file_content = get_test_file_content("compress.mod")
        .replace("\r", "") // Replace any current CR
        .replace("\n", "\r\n"); // Replace LF with CRLF

    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(gomod.module, "github.com/klauspost/compress".to_string());
}

#[test]
fn test_no_trailing_newline() {
    let file_content = get_test_file_content("tool.mod").trim().to_string(); // remove trailing newline

    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(
        gomod.tool,
        vec![
            "example.com/mymodule/cmd/mytool1",
            "example.com/mymodule/cmd/mytool2"
        ]
    );
}
