mod test_exec_common;

use test_exec_common::test_exec;

#[test]
fn brainfuck_hello_stdout() {
    let lmang_prog = "./examples/brainfuck.🆖".to_string();
    let args = ["./examples/brainfuck/hello.b".to_string()];
    let stdin = [];
    let result = test_exec(lmang_prog, &args, &stdin);

    assert_eq!(result.stdout, "Hello World!\n");
}
