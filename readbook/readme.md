# 读书之示例项目

跟着书中项目做一遍。

## The Rust Programming Language

https://doc.rust-lang.org/book/title-page.html

* `guessing_game` chapter 2
* `minigrep` chapter 12
* `webhello` chapter 20

## Command Line Applications in Rust

https://rust-cli.github.io/book/index.html
* `grrs` chapter 1

chapter 2 的一些示例放在 bin/ 目录下编译测试.

build.rs 生成 man 测试失败。
加上 build.rs 有个 bin 编译出警告.

```
warning: Error finalizing incremental compilation session directory `/home/tanshuil/github/lymslive/rustlearn/target/debug/incremental/wordcount-1wr6k6wgzfdx8/s-giqg7doiu4-19fm8mb-working`: Permission denied (os error 13)
```

但似乎也写成了 head.1 文件在某个深层目录：
../target/debug/build/grrs-d92e0865e964235b/out/head.1
