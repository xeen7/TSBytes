function main() {
    const arr = [1, 2, 3];
    let number = arr[1] * 2;
    console.log(number);
}
main();

/*
./target/debug/tsdroid jar examples/t2.ts --output /tmp/s12.jar --package com.example && java -jar /tmp/s12.jar
⚡ examples/t2.ts → JAR (/tmp/s12.jar)
✓ JAR written to /tmp/s12.jar
→ Run with: java -jar /tmp/s12.jar
Error: Unable to initialize main class com.example.App
Caused by: java.lang.VerifyError: Bad type on operand stack
Exception Details:
Location:
com/example/App.main([Ljava/lang/String;)V @83: invokevirtual
Reason:
Type double_2nd (current frame, stack[3]) is not assignable to 'java/lang/Object'
Current Frame:
bci: @83
flags: { }
locals: { '[Ljava/lang/String;', 'java/util/ArrayList' }
stack: { 'java/util/ArrayList', integer, double, double_2nd }
Bytecode:
0000000: bb00 1659 b700 1a59 1400 41b8 0006 b600
0000010: 1e57 5914 0043 b800 06b6 001e 5759 1400
0000020: 45b8 0006 b600 1e57 3a01 1901 1400 47b8
0000030: 0006 c000 4ab6 004e 1901 1400 4fb8 0006
0000040: c000 52b6 0056 b600 5cc0 005e b600 0a14
0000050: 005f 63b6 0066 57b2 006c 1901 1400 6db8
0000060: 0006 c000 70b6 0074 b600 7ac0 007c b600
0000070: 0ab8 0006 b600 8201 57b1    
 
 
*/
