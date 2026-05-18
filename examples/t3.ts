function main() {
  let arr = [1, 3, 5];
  let a = arr[0];
  let b = arr[1];
  let c = arr[2];
  a = b + c;
  console.log(a);
  b = a + c;
  console.log(b);
  c = a + b;
  console.log(c);

}
main();
/*
./target/debug/tsdroid jar examples/t3.ts --output /tmp/s12.jar --package com.example && java -jar /tmp/s12.jar
⚡ examples/t3.ts → JAR (/tmp/s12.jar)
  ✓ JAR written to /tmp/s12.jar
  → Run with: java -jar /tmp/s12.jar
Error: Unable to initialize main class com.example.App
Caused by: java.lang.VerifyError: Bad type on operand stack
Exception Details:
  Location:
    com/example/App.main([Ljava/lang/String;)V @65: astore
  Reason:
    Type double_2nd (current frame, stack[1]) is not assignable to reference type
  Current Frame:
    bci: @65
    flags: { }
    locals: { '[Ljava/lang/String;', 'java/util/ArrayList' }
    stack: { double, double_2nd }
  Bytecode:
    0000000: bb00 1659 b700 1a59 1400 41b8 0006 b600
    0000010: 1e57 5914 0043 b800 06b6 001e 5759 1400
    0000020: 45b8 0006 b600 1e57 3a01 1901 1400 47b8
    0000030: 0006 c000 4ab6 004e b600 54c0 0056 b600
    0000040: 0a3a 0219 0114 0057 b800 06c0 005a b600
    0000050: 5eb6 0064 c000 66b6 000a 3a03 1901 1400
    0000060: 67b8 0006 c000 6ab6 006e b600 74c0 0076
    0000070: b600 0a3a 0419 03b8 007c 1904 b800 82b6
    0000080: 0088 3a02 b200 8e19 02b6 0094 0157 1902
    0000090: b800 9a19 04b8 00a0 b600 a63a 03b2 00ac
    00000a0: 1903 b600 b201 5719 02b8 00b8 1903 b800
    00000b0: beb6 00c4 3a04 b200 ca19 04b6 00d0 0157
    00000c0: b1                                     

*/