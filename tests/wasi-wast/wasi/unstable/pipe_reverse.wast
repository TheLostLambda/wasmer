;; This file was generated by https://github.com/wasmerio/wasi-tests

(wasi_test "pipe_reverse.wasm"
  (assert_return (i64.const 0))
  (assert_stdout "\n")
)