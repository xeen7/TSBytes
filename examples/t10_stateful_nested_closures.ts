function main() {
  console.log("=== RUNNING STATEFUL NESTED CLOSURES TEST ===");

  // Heap-boxed outer state
  let box = { value: 10 };

  // Outer closure
  let outer = (x: number) => {
    // Inner closure capturing both `box` and outer argument `x`
    return (y: number) => {
      box.value = (box.value as number) + x + y;
      return box.value;
    };
  };

  let closure1 = outer(5); // x = 5
  let closure2 = outer(10); // x = 10

  console.log("Closure 1 first run (expect 17):");
  console.log(closure1(2)); // 10 + 5 + 2 = 17

  console.log("Closure 2 first run (expect 30):");
  console.log(closure2(3)); // 17 + 10 + 3 = 30

  console.log("Closure 1 second run (expect 40):");
  console.log(closure1(5)); // 30 + 5 + 5 = 40

  console.log("Final box value is 40:");
  console.log(box.value === 40);
  const num = 5 + 8 + 10;
  console.log(num);
  console.log(typeof num);
  console.log(typeof (5 + 8));
  console.log(typeof [1, 2, 3, 4, 5]);
  console.log("=== STATEFUL NESTED CLOSURES TEST COMPLETED SUCCESSFULLY ===");
}
main();