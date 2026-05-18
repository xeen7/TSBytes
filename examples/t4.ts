function main() {
  let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  arr[0] + 1;
  console.log(arr[0]);

}
main();
/*
./target/debug/tsdroid jar examples/t4.ts --output /tmp/test.jar --package com.example && java -jar /tmp/test.jar
⚡ examples/t4.ts → JAR (/tmp/test.jar)
  ✓ JAR written to /tmp/test.jar
  → Run with: java -jar /tmp/test.jar
Error: Unable to initialize main class com.example.App
Caused by: java.lang.VerifyError: Bad type on operand stack
Exception Details:
  Location:
    com/example/App.main([Ljava/lang/String;)V @146: pop
  Reason:
    Type double_2nd (current frame, stack[1]) is not assignable to category1 type
  Current Frame:
    bci: @146
    flags: { }
    locals: { '[Ljava/lang/String;', 'java/util/ArrayList' }
    stack: { double, double_2nd }
  Bytecode:
    0000000: bb00 1659 b700 1a59 1400 41b8 0006 b600
    0000010: 1e57 5914 0043 b800 06b6 001e 5759 1400
    0000020: 45b8 0006 b600 1e57 5914 0047 b800 06b6
    0000030: 001e 5759 1400 49b8 0006 b600 1e57 5914
    0000040: 004b b800 06b6 001e 5759 1400 4db8 0006
    0000050: b600 1e57 5914 004f b800 06b6 001e 5759
    0000060: 1400 51b8 0006 b600 1e57 5914 0053 b800
    0000070: 06b6 001e 573a 0119 0114 0055 b800 06c0
    0000080: 0058 b600 5cb6 0062 c000 64b6 000a 1400
    0000090: 6563 57b2 006c 1901 1400 6db8 0006 c000
    00000a0: 70b6 0074 b600 7ac0 007c b600 0ab8 0006
    00000b0: b600 8201 57b1                         



*/