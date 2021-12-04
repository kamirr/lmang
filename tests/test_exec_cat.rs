mod test_exec_common;

use test_exec_common::test_exec;

#[test]
fn brainfuck_hello_stdout() {
    let lmang_prog = "./examples/cat.🆖".to_string();
    let args = [];
    let stdin = ["a\n".to_string(), "b\n".to_string()];
    let result = test_exec(lmang_prog, &args, &stdin);

    assert_eq!(result.stdout, "a\nb\n");
}
