use gomod::GoMod;
use std::fs::read_to_string;
use std::path::PathBuf;

fn get_test_file_path(file_name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.join(format!("tests/data/{file_name}"))
}

fn get_test_file_content(file_name: &str) -> String {
    let path = get_test_file_path(file_name);
    read_to_string(path).unwrap()
}

#[test]
fn test_parse_on_docker_docs() {
    let file_content = get_test_file_content("docker_docs.mod");
    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(gomod.module, Some("github.com/docker/docs".to_string()))
}

#[test]
fn test_parse_on_iris() {
    let file_content = get_test_file_content("iris.mod");
    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(
        gomod.module,
        Some("github.com/kataras/iris/v12".to_string())
    )
}

#[test]
fn test_parse_on_kubernetes() {
    let file_content = get_test_file_content("kubernetes.mod");
    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(gomod.module, Some("k8s.io/kubernetes".to_string()))
}

#[test]
fn test_parse_on_prometheus() {
    let file_content = get_test_file_content("prometheus.mod");
    let gomod = file_content.parse::<GoMod>().unwrap();

    assert_eq!(
        gomod.module,
        Some("github.com/prometheus/prometheus".to_string())
    )
}
